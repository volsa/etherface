use crate::error::Error;
use crate::model::SignatureKind;
use crate::model::SignatureVisibility;
use crate::model::SignatureWithMetadata;
use lazy_static::lazy_static;
use regex::Regex;
use regex::RegexBuilder;
use semver::Version;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize)]
struct Abi {
    pub name: Option<String>,
    pub inputs: Option<Vec<AbiParameter>>,
    #[serde(rename = "type")]
    pub kind: SignatureKind,
}

#[derive(Deserialize)]
struct AbiParameter {
    #[serde(rename = "type")]
    type_: String,
}

// https://semver.npmjs.com/
#[derive(Debug, PartialEq, Eq)]
enum Condition {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equals,
    Caret, // '^' Token
    Tilde,
}

impl FromStr for Condition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(Condition::GreaterThan),
            ">=" => Ok(Condition::GreaterThanOrEqual),
            "<" => Ok(Condition::LessThan),
            "<=" => Ok(Condition::LessThanOrEqual),
            "=" => Ok(Condition::Equals),
            "^" => Ok(Condition::Caret),
            "~" => Ok(Condition::Tilde),

            _ => Err(()),
        }
    }
}

lazy_static! {
    static ref REGEX_PRAGMA: Regex = Regex::new(
        r"(?x)
            pragma                                  #
            \s+                                     # 1 to n characters between `pragma` and `solidity` keywords
            solidity                                #
            \s*                                     # 0 to n characters between `pragma solidity` and version declaration; 0 to n because e.g. `pragma solidity^0.8.0` is valid
            (                                       # Start of left handed version side, `0.7.0` in `pragma solidity >0.7.0 <= 0.8.0`
                (?P<lhs_condition>([\^~=]|[><]=?))? # (optional) Version conditions; Note that we could add everything into one bracked, i.e. [\^~><]?=?] but the RegEx crate won't recognize that `lhs_condition` for e.g. `pragma solidity 0.8.0` is None, whereas with the current pattern it does
                \s*                                 # 0 to n characters between condition and versioning
                v?                                  # (optional) match v because e.g. `pragma solidity v0.7.0` is valid
                (?P<lhs_version>[0-9\.]*)           # Versioning (X.Y.Z)
            )                                       # End of left handed version side
            \s*                                     # 0 to n characters between left handed and (optional) right handed version side
            (                                       #
                ;                                   # Semicolon indicating end of pragma
                |                                   # or
                (                                   # Start of (optional) right handed version side, `0.8.0` in `pragma solidity >0.7.0 <= 0.8.0`
                    (?P<rhs_condition>[\^~><]=?)    # Same as in left handed version side, except that a condition must be present for it to be a valid right handed version side
                    \s*                             # ^
                    \v?                             # ^
                    (?P<rhs_version>[0-9\.]*)       # ^
                )                                   # End of (optional) right handed version side
            )
        ").unwrap();

    static ref REGEX_SIGNATURE: Regex = Regex::new(
        r"(?x)                                                      # Needed symbol to annotate regex with comments (https://docs.rs/regex/latest/regex/index.html#example-replacement-with-named-capture-groups)
            (?P<kind>function|event|error)                          # Interface kind
            \s+                                                     # 1 to n whitespaces between kind and name
            (?P<name>[a-zA-Z_][a-zA-Z_0-9]*)                        # Interface name
            \s*                                                     # 0 to n whitespaces between name and parameters
            \(                                                      # Opening parameter parentheses
                (?P<params>.*?)                                     # Parameters
            \)                                                      # Closing parameter parentheses
            (                                                       # Start of **optional** visibility group
                (.*?)?                                              # Match between 0 and n characters before the visibility keyword, because sometimes there are other keywords inbetween the parameter list and the visibility keyword
                (                                                   # Match either a visibility keyword OR a semicolon / curly brace if there's no visibility keyword present (often found in event and error signatures; e.g. `event foobar(uint address);`)
                    (?P<visibility>external|public|internal|private)
                    |;
                    |\{
                )
            )?                                                      # End of **optional** visibility group (indicated by ?)
        ").unwrap();

    // The `REGEX_SIGNATURE` pattern only recognizes signatures defined within a line, as such multi-line
    // signatures won't be detected by default. To bypass this we have to remove all newlines[0] as well a
    // code-comments[1] before actually starting to extract signatures from an arbitrary Solidity file.
    // [0]
    // function foobar(
    //     address foo,
    //     uint256 bar
    // ) <other keywords>
    // => becomes `function foobar(address foo, uint256 bar) <other keywords>`
    //
    // [1]
    // function foobar(
    //     address foo,    // This is a comment
    //     uint256 bar,    /* Also a comment */
    // )
    // => becomes [0] in the first step (removed comments) and
    // `function foobar(address foo, uint256 bar) <other keywords>` in the second one (removed newlines)
    static ref REGEX_COMMENTS_AND_NEWLINES: Regex = RegexBuilder::new(
        r"(?x)
            (                   # Find
                //.*$           # Comments starting with '//' (up until the end of the line)
                |               # or
                /\*(.|\n)*?\*/  # Comments starting with '/*' (up until the end of a comment block, namely '*/')
                |               # or
                \n              # newlines if no comment was found
            )
        ").multi_line(true).build().unwrap();
}

/// Deserializes a JSON ABI file returning a vector of [`Signature`] with [`SignatureKind`] being one of
/// [`SignatureKind::Function`], [`SignatureKind::Event`] or [`SignatureKind::Error`].
pub fn from_abi(content: &str) -> Result<Vec<SignatureWithMetadata>, Error> {
    let mut signatures = Vec::new();

    for abi_entry in serde_json::from_str::<Vec<Abi>>(content).map_err(Error::ParseAbi)? {
        let kind = abi_entry.kind;

        // We're only interested in function, event and error signatures as such we can ignore everything else
        if kind != SignatureKind::Function && kind != SignatureKind::Event && kind != SignatureKind::Error {
            continue;
        }

        let name_ = match abi_entry.name {
            Some(val) => val,
            None => continue, // TODO:
        };

        let text = format!(
            "{}({})",
            name_,
            abi_entry
                .inputs
                // We sometimes (very rarely) have to deal with ABI entries with no parameter list hence we
                // return an empty vector if the unwrap fails
                .unwrap_or_else(|| Vec::with_capacity(0))
                .into_iter()
                .map(|x| x.type_)
                .collect::<Vec<String>>()
                .join(",")
        );

        let visibility = match kind {
            // ABIs don't carry any visibility information but only public and external functions are included
            // as such we can assume it's public
            SignatureKind::Function => SignatureVisibility::Public,

            // Event and error definitions do not have any visibility, see the official language grammar:
            // https://docs.soliditylang.org/en/v0.8.13/grammar.html#a4.SolidityParser.eventDefinition and
            // https://docs.soliditylang.org/en/v0.8.13/grammar.html#a4.SolidityParser.errorDefinition
            // NOTE: For the sake of our database model (visibility is a primary key), however, we auto assign
            // a visibility of public here; This should be safe / OK because events and errors are always
            // included in ABI JSON files.
            SignatureKind::Event | SignatureKind::Error => SignatureVisibility::Public,

            _ => unreachable!("Unreachable due to previous if kind != {{Function, Event, Error}} check."),
        };

        signatures.push(SignatureWithMetadata::new(text, kind, visibility));
    }

    Ok(signatures)
}

/// Parses Solidity source code returning a vector of [`Signature`] with [`SignatureKind`] being one of
/// [`SignatureKind::Function`], [`SignatureKind::Event`] or [`SignatureKind::Error`].
pub fn from_sol(content: &str) -> Vec<SignatureWithMetadata> {
    let mut signatures = Vec::new();

    let content_processed = REGEX_COMMENTS_AND_NEWLINES.replace_all(content, " ");

    for capture in REGEX_SIGNATURE.captures_iter(&content_processed) {
        let name = capture.name("name").unwrap().as_str();
        let kind: SignatureKind = capture.name("kind").unwrap().as_str().parse().unwrap();
        let params = capture.name("params").unwrap().as_str();

        let text = format!("{}({})", name, get_joined_parameter_types(params));

        let visibility = match kind {
            SignatureKind::Function => match capture.name("visibility") {
                Some(val) => val.as_str().parse::<SignatureVisibility>().unwrap(),
                None => match can_assign_public_visibility(content) {
                    Ok(val) => match val {
                        true => SignatureVisibility::Public,
                        false => continue,
                    },

                    Err(_why) => {
                        // TODO: log
                        continue;
                    }
                },
            },

            // Event- and Error-Definitions do not have any visibility according to the language grammar:
            // https://docs.soliditylang.org/en/v0.8.11/grammar.html#a4.SolidityParser.eventDefinition and
            // https://docs.soliditylang.org/en/v0.8.11/grammar.html#a4.SolidityParser.errorDefinition
            // NOTE: For the sake of our database model (visibility is a primary key), however, we auto assign
            // a visibility of public here; This should be safe / OK because events and errors are always
            // included in ABI JSON files.
            SignatureKind::Event | SignatureKind::Error => SignatureVisibility::Public,

            _ => unreachable!(
                "The `REGEX_SIGNATURE` pattern only matches against functions, events or errors
                so this should be unreachable."
            ),
        };

        signatures.push(SignatureWithMetadata::new(text, kind, visibility));
    }

    signatures
}

fn get_joined_parameter_types(raw_parameter_list: &str) -> String {
    if raw_parameter_list.trim().is_empty() {
        return "".to_string();
    }

    // Assuming raw_parameter_list equals "  address to, uint amount  "  we would first split the String at
    // each comma[1], trim each element[2], split each element at the first whitespace[3] and finally take
    // the first element of the split whitespace elements tuple[4] pushing them into a vector. The resulting
    // vector would then hold all parameter types which we can then return.
    // [1] "  address to, uint amount  "           => ["  address to", "uint amount  "]
    // [2] ["  address to", "uint amount  "]       => ["address to", "uint amount"]
    // [3] ["address to", "uint amount"]           => [("address", "to"), ("uint", "amount")]
    // [4] [("address", "to"), ("uint", "amount")] => ["address", "uint"]
    //
    // Note: Solidity supports unnamed parameters so something like "address, uint amount" where "to" is
    // omitted is valid. To detect such parameters we check whether or not we have a tuple in step 4.
    // If so the element must be ("address", "to"), if not it's simply ("address"). For more information see:
    // https://docs.soliditylang.org/en/latest/control-structures.html?highlight=anonymous#omitted-function-parameter-names
    let mut param_types = Vec::new();
    for param in raw_parameter_list.split(',') {
        match param.trim().split_once(' ') {
            Some(val) => param_types.push(val.0),

            // Unnamed parameter
            None => param_types.push(param.trim()),
        }
    }

    match param_types.len() {
        0 => panic!("This should definitely not happen"), // covered by the is_empty check at the beginning
        1 => param_types[0].to_string(),                  // param_types == ["address"]
        _ => param_types.join(","),                       // param_types == ["address", "uint",...]
    }
}

fn can_assign_public_visibility(content: &str) -> Result<bool, Error> {
    match REGEX_PRAGMA.captures(content) {
        Some(capture) => {
            // The `lhs_version` group must exists, so unwrap should be fine here
            let lhs_version = lenient_semver::parse(capture.name("lhs_version").unwrap().as_str())
                .map_err(|why| Error::ParsePragma(why.to_string()))?;

            let lhs_condition = match capture.name("lhs_condition") {
                Some(val) => val.as_str().parse::<Condition>().unwrap(),
                None => Condition::Equals, // No condition specified is the same as an equals condition
            };

            // Check if the `rhs_version` group is present
            if let Some(rhs_capture) = capture.name("rhs_version") {
                let rhs_version = lenient_semver::parse(rhs_capture.as_str())
                    .map_err(|why| Error::ParsePragma(why.to_string()))?;

                let rhs_condition = match capture.name("rhs_condition") {
                    Some(val) => val.as_str().parse::<Condition>().unwrap(),
                    None => Condition::Equals, // No condition specified is the same as an equals condition
                };

                return Ok(can_be_less_than_0_5_0(lhs_version, lhs_condition)
                    && can_be_less_than_0_5_0(rhs_version, rhs_condition));
            }

            Ok(can_be_less_than_0_5_0(lhs_version, lhs_condition))
        }

        // We're a gracious here and assume the given Solidity file is a) valid and b) has been written for
        // version < 0.5.0; This is in theory also valid because the Solidity compiler supports compiling
        // files with no `pragma solidity ...` line
        None => Ok(true),
    }
}

#[inline]
fn can_be_less_than_0_5_0(version: Version, condition: Condition) -> bool {
    let version_0_4_26 = Version::new(0, 4, 26);
    let version_0_5 = Version::new(0, 5, 0);

    if condition == Condition::GreaterThan && version >= version_0_4_26 {
        return false;
    }

    if condition == Condition::GreaterThanOrEqual && version >= version_0_5 {
        return false;
    }

    if condition == Condition::Equals && version > version_0_4_26 {
        return false;
    }

    if condition == Condition::Caret && version >= version_0_5 {
        return false;
    }

    if condition == Condition::Tilde && version >= version_0_5 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::parser::SignatureKind;
    use crate::parser::SignatureVisibility;

    #[test]
    fn from_str_signaturekind() {
        assert_eq!("function".parse::<SignatureKind>(), Ok(SignatureKind::Function));
        assert_eq!("event".parse::<SignatureKind>(), Ok(SignatureKind::Event));
        assert_eq!("error".parse::<SignatureKind>(), Ok(SignatureKind::Error));
        assert_eq!("constructor".parse::<SignatureKind>(), Ok(SignatureKind::Constructor));
        assert_eq!("fallback".parse::<SignatureKind>(), Ok(SignatureKind::Fallback));
        assert_eq!("receive".parse::<SignatureKind>(), Ok(SignatureKind::Receive));

        assert_eq!("Function".parse::<SignatureKind>(), Ok(SignatureKind::Function));
        assert_eq!("Event".parse::<SignatureKind>(), Ok(SignatureKind::Event));
        assert_eq!("Error".parse::<SignatureKind>(), Ok(SignatureKind::Error));
        assert_eq!("Constructor".parse::<SignatureKind>(), Ok(SignatureKind::Constructor));
        assert_eq!("Fallback".parse::<SignatureKind>(), Ok(SignatureKind::Fallback));
        assert_eq!("Receive".parse::<SignatureKind>(), Ok(SignatureKind::Receive));

        assert_eq!("unction".parse::<SignatureKind>(), Err(()));
    }

    #[test]
    fn from_str_signaturevisibility() {
        assert_eq!("public".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Public));
        assert_eq!("private".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Private));
        assert_eq!("external".parse::<SignatureVisibility>(), Ok(SignatureVisibility::External));
        assert_eq!("internal".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Internal));

        assert_eq!("Public".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Public));
        assert_eq!("Private".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Private));
        assert_eq!("External".parse::<SignatureVisibility>(), Ok(SignatureVisibility::External));
        assert_eq!("Internal".parse::<SignatureVisibility>(), Ok(SignatureVisibility::Internal));

        assert_eq!("int3rnal".parse::<SignatureVisibility>(), Err(()));
    }

    #[test]
    fn get_joined_parameter_types() {
        assert_eq!(&parser::get_joined_parameter_types(""), "");
        assert_eq!(&parser::get_joined_parameter_types("   "), "");
        assert_eq!(&parser::get_joined_parameter_types("address foo"), "address");
        assert_eq!(&parser::get_joined_parameter_types("   address foo "), "address");
        assert_eq!(&parser::get_joined_parameter_types("address foo, uint256[] bar"), "address,uint256[]");
        assert_eq!(
            &parser::get_joined_parameter_types("  address foo, uint256[] bar    "),
            "address,uint256[]"
        );
        assert_eq!(
            &parser::get_joined_parameter_types(" address   foo, uint256[] bar   "),
            "address,uint256[]"
        );
    }

    #[test]
    fn from_abi_all_files_without_panicing() {
        for file in std::fs::read_dir("../res/abi/").unwrap() {
            let content = std::fs::read_to_string(file.unwrap().path()).unwrap();
            parser::from_abi(&content).unwrap();
        }
    }

    #[test]
    fn from_sol_all_files_without_panicing() {
        for file in std::fs::read_dir("../res/sol/").unwrap() {
            let content = std::fs::read_to_string(file.unwrap().path()).unwrap();
            parser::from_sol(&content);
        }
    }

    #[test]
    fn from_abi_0x8bc61d005443f764d1d0d753f6ec6f9b7eae33b4() {
        #[rustfmt::skip]
        let expected_signatures = vec![
            ("Initialized()",                                                       SignatureKind::Error,       SignatureVisibility::Public),
            ("InsufficientPayment()",                                               SignatureKind::Error,       SignatureVisibility::Public),
            ("NotAuthorized()",                                                     SignatureKind::Error,       SignatureVisibility::Public),
            ("NotInitialized()",                                                    SignatureKind::Error,       SignatureVisibility::Public),
            ("NotSlicer()",                                                         SignatureKind::Error,       SignatureVisibility::Public),
            ("Claimed(address,uint256,uint256)",                                    SignatureKind::Event,       SignatureVisibility::Public),
            ("SaleClosed(address,uint256)",                                         SignatureKind::Event,       SignatureVisibility::Public),
            ("_closeSale()",                                                        SignatureKind::Function,    SignatureVisibility::Public),
            ("_setPrice(uint256)",                                                  SignatureKind::Function,    SignatureVisibility::Public),
            ("claim(uint256)",                                                      SignatureKind::Function,    SignatureVisibility::Public),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function,    SignatureVisibility::Public),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function,    SignatureVisibility::Public),
            ("releaseToCollector()",                                                SignatureKind::Function,    SignatureVisibility::Public),
            ("saleInfo()",                                                          SignatureKind::Function,    SignatureVisibility::Public),
            ("slicesLeft()",                                                        SignatureKind::Function,    SignatureVisibility::Public),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function,    SignatureVisibility::Public),
        ];

        let actual_signatures = parser::from_abi(
            &std::fs::read_to_string("../res/abi/0x8BC61d005443F764D1D0d753F6Ec6f9B7eAe33B4.json").unwrap(),
        )
        .unwrap();

        assert_eq!(actual_signatures.len(), expected_signatures.len());
        for (i, actual_signature) in actual_signatures.iter().enumerate() {
            assert_eq!(actual_signature.text, expected_signatures[i].0);
            assert_eq!(actual_signature.kind, expected_signatures[i].1);
            assert_eq!(actual_signature.visibility, expected_signatures[i].2);
        }

        // If the hash is correct for one signature then it must also be correct for all others
        assert_eq!(
            actual_signatures[0].hash,
            "5daa87a0e9463431830481fd4b6e3403442dfb9a12b9c07597e9f61d50b633c8"
        );
    }

    #[test]
    fn from_sol_0x8bc61d005443f764d1d0d753f6ec6f9b7eae33b4() {
        #[rustfmt::skip]
        let expected_signatures = vec![
            ("TransferSingle(address,address,address,uint256,uint256)",             SignatureKind::Event,      SignatureVisibility::Public),
            ("TransferBatch(address,address,address,uint256[],uint256[])",          SignatureKind::Event,      SignatureVisibility::Public),
            ("ApprovalForAll(address,address,bool)",                                SignatureKind::Event,      SignatureVisibility::Public),
            ("URI(string,uint256)",                                                 SignatureKind::Event,      SignatureVisibility::Public),
            ("balanceOf(address,uint256)",                                          SignatureKind::Function,   SignatureVisibility::External),
            ("balanceOfBatch(address[],uint256[])",                                 SignatureKind::Function,   SignatureVisibility::External),
            ("setApprovalForAll(address,bool)",                                     SignatureKind::Function,   SignatureVisibility::External),
            ("isApprovedForAll(address,address)",                                   SignatureKind::Function,   SignatureVisibility::External),
            ("safeTransferFrom(address,address,uint256,uint256,bytes)",             SignatureKind::Function,   SignatureVisibility::External),
            ("safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)",    SignatureKind::Function,   SignatureVisibility::External),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function,   SignatureVisibility::External),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function,   SignatureVisibility::External),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function,   SignatureVisibility::External),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function,   SignatureVisibility::Public),
            ("Transfer(address,address,uint256)",                                   SignatureKind::Event,      SignatureVisibility::Public),
            ("Approval(address,address,uint256)",                                   SignatureKind::Event,      SignatureVisibility::Public),
            ("ApprovalForAll(address,address,bool)",                                SignatureKind::Event,      SignatureVisibility::Public),
            ("balanceOf(address)",                                                  SignatureKind::Function,   SignatureVisibility::External),
            ("ownerOf(uint256)",                                                    SignatureKind::Function,   SignatureVisibility::External),
            ("safeTransferFrom(address,address,uint256)",                           SignatureKind::Function,   SignatureVisibility::External),
            ("transferFrom(address,address,uint256)",                               SignatureKind::Function,   SignatureVisibility::External),
            ("approve(address,uint256)",                                            SignatureKind::Function,   SignatureVisibility::External),
            ("getApproved(uint256)",                                                SignatureKind::Function,   SignatureVisibility::External),
            ("setApprovalForAll(address,bool)",                                     SignatureKind::Function,   SignatureVisibility::External),
            ("isApprovedForAll(address,address)",                                   SignatureKind::Function,   SignatureVisibility::External),
            ("safeTransferFrom(address,address,uint256,bytes)",                     SignatureKind::Function,   SignatureVisibility::External),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function,   SignatureVisibility::Public),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function,   SignatureVisibility::External),
            ("Initialized()",                                                       SignatureKind::Error,      SignatureVisibility::Public),
            ("NotInitialized()",                                                    SignatureKind::Error,      SignatureVisibility::Public),
            ("NotSlicer()",                                                         SignatureKind::Error,      SignatureVisibility::Public),
            ("NotAuthorized()",                                                     SignatureKind::Error,      SignatureVisibility::Public),
            ("InsufficientPayment()",                                               SignatureKind::Error,      SignatureVisibility::Public),
            ("Claimed(address,uint256,uint256)",                                    SignatureKind::Event,      SignatureVisibility::Public),
            ("SaleClosed(address,uint256)",                                         SignatureKind::Event,      SignatureVisibility::Public),
            ("releaseToCollector()",                                                SignatureKind::Function,   SignatureVisibility::External),
            ("_setPrice(uint256)",                                                  SignatureKind::Function,   SignatureVisibility::External),
            ("_closeSale()",                                                        SignatureKind::Function,   SignatureVisibility::External),
            ("saleInfo()",                                                          SignatureKind::Function,   SignatureVisibility::External),
            ("slicesLeft()",                                                        SignatureKind::Function,   SignatureVisibility::External),
            ("claim(uint256)",                                                      SignatureKind::Function,   SignatureVisibility::External),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function,   SignatureVisibility::External),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function,   SignatureVisibility::Public),
            ("Forward(address,uint256,address,uint256,string,bool)",                SignatureKind::Event,      SignatureVisibility::Public),
            ("terminalDirectory()",                                                 SignatureKind::Function,   SignatureVisibility::External),
            ("projectId()",                                                         SignatureKind::Function,   SignatureVisibility::External),
            ("memo()",                                                              SignatureKind::Function,   SignatureVisibility::External),
            ("SetOperator(address,address,uint256,uint256[],uint256)",              SignatureKind::Event,      SignatureVisibility::Public),
            ("permissionsOf(address,address,uint256)",                              SignatureKind::Function,   SignatureVisibility::External),
            ("hasPermission(address,address,uint256,uint256)",                      SignatureKind::Function,   SignatureVisibility::External),
            ("hasPermissions(address,address,uint256,uint256[])",                   SignatureKind::Function,   SignatureVisibility::External),
            ("setOperator(address,uint256,uint256[])",                              SignatureKind::Function,   SignatureVisibility::External),
            ("setOperators(address[],uint256[],uint256[][])",                       SignatureKind::Function,   SignatureVisibility::External),
            ("Create(uint256,address,bytes32,string,ITerminal,address)",            SignatureKind::Event,      SignatureVisibility::Public),
            ("SetHandle(uint256,bytes32,address)",                                  SignatureKind::Event,      SignatureVisibility::Public),
            ("SetUri(uint256,string,address)",                                      SignatureKind::Event,      SignatureVisibility::Public),
            ("TransferHandle(uint256,address,bytes32,bytes32,address)",             SignatureKind::Event,      SignatureVisibility::Public),
            ("ClaimHandle(address,uint256,bytes32,address)",                        SignatureKind::Event,      SignatureVisibility::Public),
            ("ChallengeHandle(bytes32,uint256,address)",                            SignatureKind::Event,      SignatureVisibility::Public),
            ("RenewHandle(bytes32,uint256,address)",                                SignatureKind::Event,      SignatureVisibility::Public),
            ("count()",                                                             SignatureKind::Function,   SignatureVisibility::External),
            ("uriOf(uint256)",                                                      SignatureKind::Function,   SignatureVisibility::External),
            ("handleOf(uint256)",                                                   SignatureKind::Function,   SignatureVisibility::External),
            ("projectFor(bytes32)",                                                 SignatureKind::Function,   SignatureVisibility::External),
            ("transferAddressFor(bytes32)",                                         SignatureKind::Function,   SignatureVisibility::External),
            ("challengeExpiryOf(bytes32)",                                          SignatureKind::Function,   SignatureVisibility::External),
            ("exists(uint256)",                                                     SignatureKind::Function,   SignatureVisibility::External),
            ("create(address,bytes32,string,ITerminal)",                            SignatureKind::Function,   SignatureVisibility::External),
            ("setHandle(uint256,bytes32)",                                          SignatureKind::Function,   SignatureVisibility::External),
            ("setUri(uint256,string)",                                              SignatureKind::Function,   SignatureVisibility::External),
            ("transferHandle(uint256,address,bytes32)",                             SignatureKind::Function,   SignatureVisibility::External),
            ("claimHandle(bytes32,address,uint256)",                                SignatureKind::Function,   SignatureVisibility::External),
            ("terminalDirectory()",                                                 SignatureKind::Function,   SignatureVisibility::External),
            ("migrationIsAllowed(ITerminal)",                                       SignatureKind::Function,   SignatureVisibility::External),
            ("pay(uint256,address,string,bool)",                                    SignatureKind::Function,   SignatureVisibility::External),
            ("addToBalance(uint256)",                                               SignatureKind::Function,   SignatureVisibility::External),
            ("allowMigration(ITerminal)",                                           SignatureKind::Function,   SignatureVisibility::External),
            ("migrate(uint256,ITerminal)",                                          SignatureKind::Function,   SignatureVisibility::External),
            ("DeployAddress(uint256,string,address)",                               SignatureKind::Event,      SignatureVisibility::Public),
            ("SetTerminal(uint256,ITerminal,address)",                              SignatureKind::Event,      SignatureVisibility::Public),
            ("SetPayerPreferences(address,address,bool)",                           SignatureKind::Event,      SignatureVisibility::Public),
            ("projects()",                                                          SignatureKind::Function,   SignatureVisibility::External),
            ("terminalOf(uint256)",                                                 SignatureKind::Function,   SignatureVisibility::External),
            ("beneficiaryOf(address)",                                              SignatureKind::Function,   SignatureVisibility::External),
            ("unstakedTicketsPreferenceOf(address)",                                SignatureKind::Function,   SignatureVisibility::External),
            ("addressesOf(uint256)",                                                SignatureKind::Function,   SignatureVisibility::External),
            ("deployAddress(uint256,string)",                                       SignatureKind::Function,   SignatureVisibility::External),
            ("setTerminal(uint256,ITerminal)",                                      SignatureKind::Function,   SignatureVisibility::External),
            ("setPayerPreferences(address,bool)",                                   SignatureKind::Function,   SignatureVisibility::External),
        ];

        let actual_signatures = parser::from_sol(
            &std::fs::read_to_string("../res/sol/0x8BC61d005443F764D1D0d753F6Ec6f9B7eAe33B4").unwrap(),
        );

        assert_eq!(actual_signatures.len(), expected_signatures.len());
        for (i, actual_signature) in actual_signatures.iter().enumerate() {
            assert_eq!(actual_signature.text, expected_signatures[i].0);
            assert_eq!(actual_signature.kind, expected_signatures[i].1);
            assert_eq!(actual_signature.visibility, expected_signatures[i].2);
        }

        // If the hash is correct for one signature then it must also be correct for all others
        assert_eq!(
            actual_signatures[0].hash,
            "c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62"
        );
    }

    #[test]
    fn from_sol_custom_signatures() {
        let code = r#"
        function supportsInterface(bytes4 interfaceId) external view returns (bool);

        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        error Recv(address indexed from, address indexed to, uint256 indexed tokenId);

        function safeTransferFrom(
            address from,
            address to,
            uint256 tokenId
        ) external;

        function toHexString(uint256 value, uint256 length) internal pure returns (string memory) {
            ...
        }

        function functionCall(
            address target,
            bytes memory data,
            string memory errorMessage
        ) internal returns (bytes memory) {
            ...
        }

        function _transfer(
            address from,
            address to,
            uint256 tokenId
        ) internal virtual {
            ...
        }

        function tokenURI(uint256 tokenId)
        public
        view
        virtual
        override
        returns (string memory)
        {
            ...
        }

        function doesntWorkButNowDoesBecauseItsFixedYay(
            address from,   // this is a comment
            uint256 id     /* also a comment */
        ) internal {

        }
        "#;

        let signatures = parser::from_sol(&code);
        assert_eq!(signatures[0].text, "supportsInterface(bytes4)");
        assert_eq!(signatures[0].kind, SignatureKind::Function);
        assert_eq!(signatures[0].visibility, SignatureVisibility::External);

        assert_eq!(signatures[1].text, "Transfer(address,address,uint256)");
        assert_eq!(signatures[1].kind, SignatureKind::Event);
        assert_eq!(signatures[1].visibility, SignatureVisibility::Public);

        assert_eq!(signatures[2].text, "Recv(address,address,uint256)");
        assert_eq!(signatures[2].kind, SignatureKind::Error);
        assert_eq!(signatures[2].visibility, SignatureVisibility::Public);

        assert_eq!(signatures[3].text, "safeTransferFrom(address,address,uint256)");
        assert_eq!(signatures[3].kind, SignatureKind::Function);
        assert_eq!(signatures[3].visibility, SignatureVisibility::External);

        assert_eq!(signatures[4].text, "toHexString(uint256,uint256)");
        assert_eq!(signatures[4].kind, SignatureKind::Function);
        assert_eq!(signatures[4].visibility, SignatureVisibility::Internal);

        assert_eq!(signatures[5].text, "functionCall(address,bytes,string)");
        assert_eq!(signatures[5].kind, SignatureKind::Function);
        assert_eq!(signatures[5].visibility, SignatureVisibility::Internal);

        assert_eq!(signatures[6].text, "_transfer(address,address,uint256)");
        assert_eq!(signatures[6].kind, SignatureKind::Function);
        assert_eq!(signatures[6].visibility, SignatureVisibility::Internal);

        assert_eq!(signatures[7].text, "tokenURI(uint256)");
        assert_eq!(signatures[7].kind, SignatureKind::Function);
        assert_eq!(signatures[7].visibility, SignatureVisibility::Public);

        assert_eq!(signatures[8].text, "doesntWorkButNowDoesBecauseItsFixedYay(address,uint256)");
        assert_eq!(signatures[8].kind, SignatureKind::Function);
        assert_eq!(signatures[8].visibility, SignatureVisibility::Internal);
    }

    #[test]
    fn from_sol_signature_visibility() {
        let code = r#"
        function signatureA01() internal pure returns (uint256) {}
        function signatureA02() pure internal returns (uint256) {}
        function signatureB01(uint256 x) external pure {}
        function signatureB02(uint256 x) pure external {}
        function signatureC01(uint256 x, addresss y) public pure {}
        function signatureC02(uint256 x, address y) pure public {}
        function signatureD01(uint256 x, addresss y) private pure {}
        function signatureD02(uint256 x, address y) pure private {}
        "#;

        let signatures = parser::from_sol(code);
        assert_eq!(signatures[0].visibility, SignatureVisibility::Internal);
        assert_eq!(signatures[1].visibility, SignatureVisibility::Internal);

        assert_eq!(signatures[2].visibility, SignatureVisibility::External);
        assert_eq!(signatures[3].visibility, SignatureVisibility::External);

        assert_eq!(signatures[4].visibility, SignatureVisibility::Public);
        assert_eq!(signatures[5].visibility, SignatureVisibility::Public);

        assert_eq!(signatures[6].visibility, SignatureVisibility::Private);
        assert_eq!(signatures[7].visibility, SignatureVisibility::Private);
    }

    #[test]
    fn from_sol_assign_visibility() {
        let code = r#"
            pragma solidity 0.4.13;

            function foobar() {}            // public visibility should be assigned here
            function foobar() public {}     // visibility should be untouched here
            function foobar() private {}    // ^
            function foobar() internal {}   // ^
            function foobar() external {}   // ^
        "#;

        let signatures = crate::parser::from_sol(code);
        assert_eq!(signatures[0].visibility, SignatureVisibility::Public);
        assert_eq!(signatures[1].visibility, SignatureVisibility::Public);
        assert_eq!(signatures[2].visibility, SignatureVisibility::Private);
        assert_eq!(signatures[3].visibility, SignatureVisibility::Internal);
        assert_eq!(signatures[4].visibility, SignatureVisibility::External);
    }

    #[test]
    fn pragma() {
        let pragmas_valid = vec![
            // "pragma solidity ^0.4.0 || ^0.5.0 || ^0.6.0;",
            "pragma solidity 0.4.10;",
            "pragma solidity 0.4.13;",
            "pragma solidity 0.4.14;",
            "pragma solidity 0.4.16;",
            "pragma solidity 0.4.17;",
            "pragma solidity 0.4.19;",
            "pragma solidity 0.4.24;",
            "pragma solidity 0.4.25;",
            "pragma solidity 0.4.8;",
            "pragma solidity 0.4.9;",
            "pragma solidity < 0.4.0;",
            "pragma solidity < 0.5;",
            "pragma solidity < 0.6;",
            "pragma solidity < 1;",
            "pragma solidity <0.9.0;",
            "pragma solidity <= 0.4.0;",
            "pragma solidity <= 0.4;",
            "pragma solidity <= 0.8.0;",
            "pragma solidity <= 0;",
            "pragma solidity <=0.4.20;",
            "pragma solidity <=0.5.16;",
            "pragma solidity <=0.6.7;",
            "pragma solidity <=0.7.0;",
            "pragma solidity <=0.7.2;",
            "pragma solidity > 0.4.0 < 0.8.0;",
            "pragma solidity > 0.4.0;",
            "pragma solidity > 0.4.4;",
            "pragma solidity > 0.4;",
            "pragma solidity >0.4.0 <0.7.0;",
            "pragma solidity >0.4.0 <0.8.0;",
            "pragma solidity >0.4.0 <=0.7.0;",
            "pragma solidity >0.4.0 <=0.9.0;",
            "pragma solidity >0.4.13 <=0.7.5;",
            "pragma solidity >0.4.16;",
            "pragma solidity >0.4.18 <0.5.0;",
            "pragma solidity >0.4.19;",
            "pragma solidity >0.4.21 <0.6.0;",
            "pragma solidity >0.4.22 <0.6.0;",
            "pragma solidity >0.4.25;",
            "pragma solidity >0.4;",
            "pragma solidity >= 0.4.0 < 0.7.3;",
            "pragma solidity >= 0.4.11 < 0.5;",
            "pragma solidity >= 0.4.19 < 0.9.0;",
            "pragma solidity >= 0.4.19;",
            "pragma solidity >= 0.4.21 <0.6.0;",
            "pragma solidity >= 0.4.22 < 0.5;",
            "pragma solidity >= 0.4.22 < 0.7.0;",
            "pragma solidity >= 0.4.22 < 0.70;",
            "pragma solidity >= 0.4.22 < 0.8.0;",
            "pragma solidity >= 0.4.22;",
            "pragma solidity >= 0.4.25 < 0.6.0;",
            "pragma solidity >= 0.4.3 < 0.9.0;",
            "pragma solidity >= 0.4.5<0.60;",
            "pragma solidity >= 0.4;",
            "pragma solidity >=0.1.1 <=0.5.0;",
            "pragma solidity >=0.4 <0.9;",
            "pragma solidity >=0.4 <=0.4;",
            "pragma solidity >=0.4.0 < 0.7.0;",
            "pragma solidity >=0.4.0 < 0.8.0;",
            "pragma solidity >=0.4.0 < 0.9.0;",
            "pragma solidity >=0.4.0 <0.4.8;",
            "pragma solidity >=0.4.0 <0.7.0;",
            "pragma solidity >=0.4.0 <0.7.1;",
            "pragma solidity >=0.4.0 <0.8.0;",
            "pragma solidity >=0.4.0 <0.9.0;",
            "pragma solidity >=0.4.0 <=0.6.0;",
            "pragma solidity >=0.4.0 <=0.8.0;",
            "pragma solidity >=0.4.10 <0.6.0;",
            "pragma solidity >=0.4.11 <0.5.0;",
            "pragma solidity >=0.4.13;",
            "pragma solidity >=0.4.14;",
            "pragma solidity >=0.4.15 < 0.8.0;",
            "pragma solidity >=0.4.15 <0.5.0;",
            "pragma solidity >=0.4.15 <0.6.0;",
            "pragma solidity >=0.4.15;",
            "pragma solidity >=0.4.16 < 0.9.0;",
            "pragma solidity >=0.4.16 <0.8.0;",
            "pragma solidity >=0.4.16;",
            "pragma solidity >=0.4.17 ;",
            "pragma solidity >=0.4.17 <0.7.0;",
            "pragma solidity >=0.4.18 <0.6.0;",
            "pragma solidity >=0.4.18 <0.7.0;",
            "pragma solidity >=0.4.19 <0.6.0;",
            "pragma solidity >=0.4.19 <0.7.0;",
            "pragma solidity >=0.4.2 ;",
            "pragma solidity >=0.4.2 <0.6.0;",
            "pragma solidity >=0.4.2 <0.6.1;",
            "pragma solidity >=0.4.20 <0.6.1;",
            "pragma solidity >=0.4.21 <0.6.0;",
            "pragma solidity >=0.4.21 <0.6.1;",
            "pragma solidity >=0.4.21 <0.71.0;",
            "pragma solidity >=0.4.21 <=0.6.0;",
            "pragma solidity >=0.4.21 <=0.6.12;",
            "pragma solidity >=0.4.21 <=0.6.6;",
            "pragma solidity >=0.4.21 <=0.6.7;",
            "pragma solidity >=0.4.21 <=0.7.2;",
            "pragma solidity >=0.4.21 <=0.7.5;",
            "pragma solidity >=0.4.21;",
            "pragma solidity >=0.4.22 < 0.7.0;",
            "pragma solidity >=0.4.22 <0.6.0;",
            "pragma solidity >=0.4.22 <0.9.0;",
            "pragma solidity >=0.4.22 <=0.7.0;",
            "pragma solidity >=0.4.22 <=0.7.5;",
            "pragma solidity >=0.4.22<0.6.0;",
            "pragma solidity >=0.4.23 < 0.6.0;",
            "pragma solidity >=0.4.24  <0.6.0;",
            "pragma solidity >=0.4.24 < 0.7.0;",
            "pragma solidity >=0.4.24 <0.5.0;",
            "pragma solidity >=0.4.24 <0.6.0;",
            "pragma solidity >=0.4.24 <0.6.11;",
            "pragma solidity >=0.4.24 <0.7.0;",
            "pragma solidity >=0.4.24 <= 0.5.16;",
            "pragma solidity >=0.4.25 < 6.0;",
            "pragma solidity >=0.4.25 <=0.6.10;",
            "pragma solidity >=0.4.25 <=0.7.0;",
            "pragma solidity >=0.4.26 <0.6.0;",
            "pragma solidity >=0.4.6 <0.8.0;",
            "pragma solidity >=0.4.6;",
            "pragma solidity >=0.4.8 <0.8.0 ;",
            "pragma solidity >=0.4.8 <0.8.0;",
            "pragma solidity ^ 0.4.0;",
            "pragma solidity ^ 0.4.13;",
            "pragma solidity ^ 0.4.15;",
            "pragma solidity ^ 0.4.17;",
            "pragma solidity ^ 0.4.21;",
            "pragma solidity ^ 0.4.24;",
            "pragma solidity ^ 0.4.25;",
            "pragma solidity ^ 0.4.26;",
            "pragma solidity ^ 0.4.4;",
            "pragma solidity ^ 0.4.8;",
            "pragma solidity ^ 0.4.8;",
            "pragma solidity ^0.4.0;",
            "pragma solidity ^0.4.10;",
            "pragma solidity ^0.4.12;",
            "pragma solidity ^0.4.12;",
            "pragma solidity ^0.4.13;",
            "pragma solidity ^0.4.16;",
            "pragma solidity ^0.4.17 < 0.6.12;",
            "pragma solidity ^0.4.17 <0.6.12;",
            "pragma solidity ^0.4.18;",
            "pragma solidity ^0.4.20;",
            "pragma solidity ^0.4.21 < 0.7.0;",
            "pragma solidity ^0.4.21 <0.6.12;",
            "pragma solidity ^0.4.21;",
            "pragma solidity ^0.4.21;",
            "pragma solidity ^0.4.21;",
            "pragma solidity ^0.4.23;",
            "pragma solidity ^0.4.23;",
            "pragma solidity ^0.4.24 ;",
            "pragma solidity ^0.4.24;",
            "pragma solidity ^0.4.26;",
            "pragma solidity ^0.4.6;",
            "pragma solidity ^0.4.7;",
            "pragma solidity v0.4.0;",
            "pragma solidity v0.4;",
            "pragma solidity ~0.4.19;",
            "pragma solidity ~0.4.21;",
            "pragma solidity ~0.4.24;",
        ];

        let pragmas_invalid = vec![
            // "pragma solidity ~0.4.24 >=0.5;",
            // "pragma solidity 0.7.x;",
            // "pragma solidity > 0;",
            // "pragma solidity >= 0;",
            // "pragma solidity >=0 <=1;",
            // "pragma solidity ^0;",
            // "pragma solidity ^;",
            // "pragma solidity v0;",
            "pragma solidity ^0.6.0;",
            "pragma solidity 0.5.0;",
            "pragma solidity 0.5.10;",
            "pragma solidity 0.5.12;",
            "pragma solidity 0.5.14;",
            "pragma solidity 0.5.15;",
            "pragma solidity 0.5.16;",
            "pragma solidity 0.5.1;",
            "pragma solidity 0.5.3;",
            "pragma solidity 0.5.4;",
            "pragma solidity 0.5.5;",
            "pragma solidity 0.5.6;",
            "pragma solidity 0.5.7;",
            "pragma solidity 0.5.7;",
            "pragma solidity 0.6.0;",
            "pragma solidity 0.6.10;",
            "pragma solidity 0.6.1;",
            "pragma solidity 0.6.5;",
            "pragma solidity 0.6.9;",
            "pragma solidity 0.7.12;",
            "pragma solidity 0.7.5 <0.8.0;",
            "pragma solidity 0.7.6;",
            "pragma solidity 0.7;",
            "pragma solidity 0.8 <= 0.9;",
            "pragma solidity 0.8.2;",
            "pragma solidity 0.8.5;",
            "pragma solidity 0.8.7;",
            "pragma solidity = 0.5.2;",
            "pragma solidity = 0.6.0;",
            "pragma solidity = 0.6.11;",
            "pragma solidity = 0.7.5;",
            "pragma solidity =0.5.0;",
            "pragma solidity =0.5.1;",
            "pragma solidity =0.5.3;",
            "pragma solidity =0.6.11;",
            "pragma solidity =0.6.4;",
            "pragma solidity =0.6.7;",
            "pragma solidity =0.7.2;",
            "pragma solidity =0.7.6;",
            "pragma solidity =0.8.0;",
            "pragma solidity > 0.5.2 < 0.6.0;",
            "pragma solidity > 0.6.0 < 0.7.0;",
            "pragma solidity > 0.6.0;",
            "pragma solidity > 0.6.1 < 0.7.0;",
            "pragma solidity > 0.6.1 <= 0.7.0;",
            "pragma solidity > 0.6.99 < 0.8.0;",
            "pragma solidity >0.5.0 <0.9.0;",
            "pragma solidity >0.5.10 <0.8.0;",
            "pragma solidity >0.5.17;",
            "pragma solidity >0.5.2 <0.8.0;",
            "pragma solidity >0.5.4;",
            "pragma solidity >0.6.1 <=0.7.0;",
            "pragma solidity >0.6.10 <0.7;",
            "pragma solidity >0.6.10;",
            "pragma solidity >0.6.12;",
            "pragma solidity >0.6.9 <0.8.0;",
            "pragma solidity >0.7.0 <0.9.0;",
            "pragma solidity >0.7.1;",
            "pragma solidity >0.7.2;",
            "pragma solidity >= 0.5 < 0.6;",
            "pragma solidity >= 0.5.0 < 0.6.0;",
            "pragma solidity >= 0.5.0 < 0.6.0;",
            "pragma solidity >= 0.5.0 < 0.8;",
            "pragma solidity >= 0.5.0 < 0.9.0;",
            "pragma solidity >= 0.5.0 <0.6.0;",
            "pragma solidity >= 0.5.10 < 0.7.0;",
            "pragma solidity >= 0.5.10;",
            "pragma solidity >= 0.5.17;",
            "pragma solidity >= 0.5.3 < 0.7.3;",
            "pragma solidity >= 0.5.3;",
            "pragma solidity >= 0.5.7;",
            "pragma solidity >= 0.5;",
            "pragma solidity >= 0.6 <0.8;",
            "pragma solidity >= 0.6.0 < 0.8;",
            "pragma solidity >= 0.6.0 <= 0.7.0;",
            "pragma solidity >= 0.6.12;",
            "pragma solidity >= 0.6.5;",
            "pragma solidity >= 0.6.7 <0.7.0;",
            "pragma solidity >= 0.7.0 < 0.8;",
            "pragma solidity >= 0.7.0 <0.8.0;",
            "pragma solidity >= 0.7.0 <0.9.0;",
            "pragma solidity >= 0.7.5;",
            "pragma solidity >= 0.7.6 < 0.8.0;",
            "pragma solidity >= 0.8;",
            "pragma solidity >=0.5 < 0.8;",
            "pragma solidity >=0.5.0 < 0.7.0 ;",
            "pragma solidity >=0.5.0 < 0.8.0;",
            "pragma solidity >=0.5.0 <0.6.9;",
            "pragma solidity >=0.5.0 <0.7.5;",
            "pragma solidity >=0.5.0 <= 0.6.2;",
            "pragma solidity >=0.5.1 < 0.6.0;",
            "pragma solidity >=0.5.1 <0.7.0;",
            "pragma solidity >=0.5.10 <0.8.0;",
            "pragma solidity >=0.5.10;",
            "pragma solidity >=0.5.11 <0.6.2;",
            "pragma solidity >=0.5.12 <0.8.0;",
            "pragma solidity >=0.5.12 <= 0.6.0;",
            "pragma solidity >=0.5.13 <0.7.3;",
            "pragma solidity >=0.5.14 <0.7.4;",
            "pragma solidity >=0.5.15 <0.6.8;",
            "pragma solidity >=0.5.16 <0.7.0;",
            "pragma solidity >=0.5.16 <0.7.1;",
            "pragma solidity >=0.5.16 <0.8.0;",
            "pragma solidity >=0.5.17 <0.7.0;",
            "pragma solidity >=0.5.17 <0.8.5;",
            "pragma solidity >=0.5.1<0.8.0;",
            "pragma solidity >=0.5.2 <0.5.4;",
            "pragma solidity >=0.5.3 <0.7.0;",
            "pragma solidity >=0.5.3 <=0.5.8;",
            "pragma solidity >=0.5.3;",
            "pragma solidity >=0.5.3<0.6.0;",
            "pragma solidity >=0.5.5;",
            "pragma solidity >=0.5.7 <0.7.0;",
            "pragma solidity >=0.5.8 < 0.6.0;",
            "pragma solidity >=0.5.8 <0.9.0;",
            "pragma solidity >=0.5;",
            "pragma solidity >=0.6.0 <0.7.3;",
            "pragma solidity >=0.6.0 <0.7.5;",
            "pragma solidity >=0.6.0 <0.7;",
            "pragma solidity >=0.6.0 <0.8.4;",
            "pragma solidity >=0.6.0 <0.9.0;",
            "pragma solidity >=0.6.0 <0.9.0;",
            "pragma solidity >=0.6.10 <0.7;",
            "pragma solidity >=0.6.11 <0.9.0;",
            "pragma solidity >=0.6.12 <0.9.0;",
            "pragma solidity >=0.6.2 <0.9.0;",
            "pragma solidity >=0.6.3;",
            "pragma solidity >=0.6.4 <0.8.5;",
            "pragma solidity >=0.6.6 <0.9.0;",
            "pragma solidity >=0.6.8 <0.7.0;",
            "pragma solidity >=0.6.8 <0.9.0;",
            "pragma solidity >=0.6.9;",
            "pragma solidity >=0.7 < 0.8;",
            "pragma solidity >=0.7 <0.9;",
            "pragma solidity >=0.7.0 < 0.8.0;",
            "pragma solidity >=0.7.0 <0.8.0;",
            "pragma solidity >=0.7.0 <0.8.2;",
            "pragma solidity >=0.7.0;",
            "pragma solidity >=0.7.1 <0.8.0;",
            "pragma solidity >=0.7.1 <0.9.0;",
            "pragma solidity >=0.7.3;",
            "pragma solidity >=0.7.5 <8.0.0;",
            "pragma solidity >=0.7.5;",
            "pragma solidity >=0.8.0 <0.9.0 >=0.8.6 <0.9.0;",
            "pragma solidity >=0.8.0;",
            "pragma solidity >=0.8.2 <0.9.0;",
            "pragma solidity >=0.8.3;",
            "pragma solidity >=0.8.6;",
            "pragma solidity ^ 0.5.0;",
            "pragma solidity ^ 0.5.16;",
            "pragma solidity ^ 0.5.17;",
            "pragma solidity ^ 0.5.7;",
            "pragma solidity ^ 0.5.7;",
            "pragma solidity ^ 0.5.9;",
            "pragma solidity ^ 0.6.0 ;",
            "pragma solidity ^ 0.6.0;",
            "pragma solidity ^ 0.6.10;",
            "pragma solidity ^ 0.6.1;",
            "pragma solidity ^0.5 <0.6.0;",
            "pragma solidity ^0.5.0 ;",
            "pragma solidity ^0.5.0 <0.6.0;",
            "pragma solidity ^0.5.0 <0.7.0;",
            "pragma solidity ^0.5.0 <6.0.0;",
            "pragma solidity ^0.5.0;",
            "pragma solidity ^0.5.0;",
            "pragma solidity ^0.5.0;",
            "pragma solidity ^0.5.10;",
            "pragma solidity ^0.5.11;",
            "pragma solidity ^0.5.13;",
            "pragma solidity ^0.5.15;",
            "pragma solidity ^0.5.15;",
            "pragma solidity ^0.5.16;",
            "pragma solidity ^0.5.16;",
            "pragma solidity ^0.5.3;",
            "pragma solidity ^0.5.4.0;",
            "pragma solidity ^0.5.4;",
            "pragma solidity ^0.5.5 <0.5.8;",
            "pragma solidity ^0.5.7;",
            "pragma solidity ^0.5.8;",
            "pragma solidity ^0.5.8;",
            "pragma solidity ^0.6.1;",
            "pragma solidity ^0.6.4;",
            "pragma solidity ^0.6.6;",
            "pragma solidity ^0.6;",
            "pragma solidity ^0.7.0 ;",
            "pragma solidity ^0.7.4;",
            "pragma solidity ^0.7.5;",
            "pragma solidity ^0.7.6;",
            "pragma solidity ^0.8.0 <0.9.0;",
            "pragma solidity ^0.8.2;",
            "pragma solidity ^0.8.6;",
            "pragma solidity ^4.4.0;",
            "pragma solidity=0.6.12;",
            "pragma solidity^0.5.10;",
            "pragma solidity^0.5.17;",
            "pragma solidity^0.6.0;",
            "pragma solidity^0.8.0;",
        ];

        for pragma in pragmas_valid {
            assert_eq!(crate::parser::can_assign_public_visibility(pragma).unwrap(), true);
        }

        for pragma in pragmas_invalid {
            assert_eq!(crate::parser::can_assign_public_visibility(pragma).unwrap(), false);
        }
    }
}
