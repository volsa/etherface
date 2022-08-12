//! (RegEx) Parser responsible for extracting function, event and error signatures from arbitrary Solidity / 
//! ABI files.
//! 
//! For Solidity files the parser works by using RegEx patterns to extract signatures such as 
//! `function foobar(uint256 val) external payable {` from e.g.
//! ```
//! pragma Solidity 0.8.14; 
//! contract Example {
//!     // ...
//!     function foobar(uint256 val) external payable {
//!         // ...
//!     }
//!     // ...
//! }
//! ```
//! which can then be further used to extract the signature type (`function` in this case) as well as the 
//! canonical signature (`foobar(uint256)` in this case). These extracted informations are then stored inside
//! a [`SignatureWithMetadata`] struct and returned to the caller.
//! 
//! For ABI (= JSON) files the parser simply uses serde to deserialize the content and assemble all extracted
//! data to form the canonical signature.

use crate::error::Error;
use crate::model::SignatureKind;
use crate::model::SignatureWithMetadata;
use lazy_static::lazy_static;
use regex::Regex;
use regex::RegexBuilder;
use serde::Deserialize;

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

    static ref REGEX_PARAMETER_TYPES: Regex = Regex::new(
        r"(?x)
            (   
                (
                    address|
                    bool|
                    string|
                    bytes(\d{0,3})?|
                    int(\d{0,3})?|
                    uint(\d{0,3})?|
                    fixed|
                    ufixed
                )
            (\[\d*\])*)                 # (optional) Array declaration (0 - * times)
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

/// Returns a list of [`SignatureWithMetadata`] extracted from a JSON ABI file.
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
            None => continue, // Can't create a signature if no name is present (duh)
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

        signatures.push(SignatureWithMetadata::new(text, kind, true));
    }

    Ok(signatures)
}

/// Returns a list of [`SignatureWithMetadata`] extracted from a Solidity file.
pub fn from_sol(content: &str) -> Vec<SignatureWithMetadata> {
    let mut signatures = Vec::new();

    let content_processed = REGEX_COMMENTS_AND_NEWLINES.replace_all(content, " ");

    for capture in REGEX_SIGNATURE.captures_iter(&content_processed) {
        let name = capture.name("name").unwrap().as_str();
        let kind: SignatureKind = capture.name("kind").unwrap().as_str().parse().unwrap();

        let (text, is_valid) = match get_split_parameter_list(capture.name("params").unwrap().as_str()) {
            Some(list) => (format!("{name}({})", list.join(",")), parameter_types_are_valid(&list)),
            None => (format!("{name}()"), true),
        };

        // let is_valid = parameter_types_are_valid(&params);
        // let text = format!("{}({})", name, get_joined_parameter_types(params));

        signatures.push(SignatureWithMetadata::new(text, kind, is_valid));
    }

    signatures
}

/// Checks whether or not the given parameter type is valid, i.e. not an user defined type (see 
/// <https://blog.soliditylang.org/2021/09/27/user-defined-value-types/>).
fn parameter_types_are_valid(params: &Vec<String>) -> bool {
    for param in params {
        if !REGEX_PARAMETER_TYPES.is_match(param) {
            if param.is_empty() {
                continue;
            }

            return false;
        }
    }

    true
}

/// Converts and returns a parameter list such as `uint foo, uint bar` to a vector of `[uint, uint]`.
fn get_split_parameter_list(raw_parameter_list: &str) -> Option<Vec<String>> {
    if raw_parameter_list.trim().is_empty() {
        return None;
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
            Some(val) => param_types.push(val.0.to_string()),

            // Unnamed parameter
            None => param_types.push(param.trim().to_string()),
        }
    }

    Some(param_types)
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::parser::SignatureKind;

    use super::parameter_types_are_valid;

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
    #[rustfmt::skip]
    fn get_joined_parameter_types() {
        assert_eq!(parser::get_split_parameter_list(""), None);
        assert_eq!(parser::get_split_parameter_list("   "), None);
        assert_eq!(parser::get_split_parameter_list("address foo"), Some(vec!["address".into()]));
        assert_eq!(parser::get_split_parameter_list("   address foo "), Some(vec!["address".into()]));
        assert_eq!(parser::get_split_parameter_list("address foo, uint256[] bar"), Some(vec!["address".into(),"uint256[]".into()]));
        assert_eq!(parser::get_split_parameter_list("  address foo, uint256[] bar    "), Some(vec!["address".into(),"uint256[]".into()]));
        assert_eq!(parser::get_split_parameter_list(" address   foo, uint256[] bar   "), Some(vec!["address".into(),"uint256[]".into()]));
    }

    #[test]
    #[rustfmt::skip]
    fn check_validity() {
        let valid_param_types: Vec<Vec<String>> = vec![
            vec!["".into()],
            "uint256,address,uint8,uint8,uint256".split(",").map(str::to_string).collect(),
            "address,address[]".split(",").map(str::to_string).collect(),
            "address[],address[],uint256[],uint256[],uint256[],uint256".split(",").map(str::to_string).collect(),
            "address[],uint256[],uint256[][]".split(",").map(str::to_string).collect(),
        ];

        let invalid_param_types: Vec<Vec<String>> = vec![
            "IUniswapV2Pair,uint256,uint256,uint256,address".split(",").map(str::to_string).collect(),
            "ISolidlyLens.PositionVe[]".split(",").map(str::to_string).collect(),
            "uint256,address,bytes32,string,ITerminal,address".split(",").map(str::to_string).collect(),
        ];

        for params in valid_param_types {
            assert_eq!(parameter_types_are_valid(&params), true);
        }
        
        for params in invalid_param_types {
            assert_eq!(parameter_types_are_valid(&params), false);
        }
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
            ("Initialized()",                                                       SignatureKind::Error),
            ("InsufficientPayment()",                                               SignatureKind::Error),
            ("NotAuthorized()",                                                     SignatureKind::Error),
            ("NotInitialized()",                                                    SignatureKind::Error),
            ("NotSlicer()",                                                         SignatureKind::Error),
            ("Claimed(address,uint256,uint256)",                                    SignatureKind::Event),
            ("SaleClosed(address,uint256)",                                         SignatureKind::Event),
            ("_closeSale()",                                                        SignatureKind::Function),
            ("_setPrice(uint256)",                                                  SignatureKind::Function),
            ("claim(uint256)",                                                      SignatureKind::Function),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function),
            ("releaseToCollector()",                                                SignatureKind::Function),
            ("saleInfo()",                                                          SignatureKind::Function),
            ("slicesLeft()",                                                        SignatureKind::Function),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function),
        ];

        let actual_signatures = parser::from_abi(
            &std::fs::read_to_string("../res/abi/0x8BC61d005443F764D1D0d753F6Ec6f9B7eAe33B4.json").unwrap(),
        )
        .unwrap();

        assert_eq!(actual_signatures.len(), expected_signatures.len());
        for (i, actual_signature) in actual_signatures.iter().enumerate() {
            assert_eq!(actual_signature.text, expected_signatures[i].0);
            assert_eq!(actual_signature.kind, expected_signatures[i].1);
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
            ("TransferSingle(address,address,address,uint256,uint256)",             SignatureKind::Event),
            ("TransferBatch(address,address,address,uint256[],uint256[])",          SignatureKind::Event),
            ("ApprovalForAll(address,address,bool)",                                SignatureKind::Event),
            ("URI(string,uint256)",                                                 SignatureKind::Event),
            ("balanceOf(address,uint256)",                                          SignatureKind::Function),
            ("balanceOfBatch(address[],uint256[])",                                 SignatureKind::Function),
            ("setApprovalForAll(address,bool)",                                     SignatureKind::Function),
            ("isApprovedForAll(address,address)",                                   SignatureKind::Function),
            ("safeTransferFrom(address,address,uint256,uint256,bytes)",             SignatureKind::Function),
            ("safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)",    SignatureKind::Function),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function),
            ("Transfer(address,address,uint256)",                                   SignatureKind::Event),
            ("Approval(address,address,uint256)",                                   SignatureKind::Event),
            ("ApprovalForAll(address,address,bool)",                                SignatureKind::Event),
            ("balanceOf(address)",                                                  SignatureKind::Function),
            ("ownerOf(uint256)",                                                    SignatureKind::Function),
            ("safeTransferFrom(address,address,uint256)",                           SignatureKind::Function),
            ("transferFrom(address,address,uint256)",                               SignatureKind::Function),
            ("approve(address,uint256)",                                            SignatureKind::Function),
            ("getApproved(uint256)",                                                SignatureKind::Function),
            ("setApprovalForAll(address,bool)",                                     SignatureKind::Function),
            ("isApprovedForAll(address,address)",                                   SignatureKind::Function),
            ("safeTransferFrom(address,address,uint256,bytes)",                     SignatureKind::Function),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function),
            ("supportsInterface(bytes4)",                                           SignatureKind::Function),
            ("Initialized()",                                                       SignatureKind::Error),
            ("NotInitialized()",                                                    SignatureKind::Error),
            ("NotSlicer()",                                                         SignatureKind::Error),
            ("NotAuthorized()",                                                     SignatureKind::Error),
            ("InsufficientPayment()",                                               SignatureKind::Error),
            ("Claimed(address,uint256,uint256)",                                    SignatureKind::Event),
            ("SaleClosed(address,uint256)",                                         SignatureKind::Event),
            ("releaseToCollector()",                                                SignatureKind::Function),
            ("_setPrice(uint256)",                                                  SignatureKind::Function),
            ("_closeSale()",                                                        SignatureKind::Function),
            ("saleInfo()",                                                          SignatureKind::Function),
            ("slicesLeft()",                                                        SignatureKind::Function),
            ("claim(uint256)",                                                      SignatureKind::Function),
            ("onERC1155Received(address,address,uint256,uint256,bytes)",            SignatureKind::Function),
            ("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)",   SignatureKind::Function),
            ("Forward(address,uint256,address,uint256,string,bool)",                SignatureKind::Event),
            ("terminalDirectory()",                                                 SignatureKind::Function),
            ("projectId()",                                                         SignatureKind::Function),
            ("memo()",                                                              SignatureKind::Function),
            ("SetOperator(address,address,uint256,uint256[],uint256)",              SignatureKind::Event),
            ("permissionsOf(address,address,uint256)",                              SignatureKind::Function),
            ("hasPermission(address,address,uint256,uint256)",                      SignatureKind::Function),
            ("hasPermissions(address,address,uint256,uint256[])",                   SignatureKind::Function),
            ("setOperator(address,uint256,uint256[])",                              SignatureKind::Function),
            ("setOperators(address[],uint256[],uint256[][])",                       SignatureKind::Function),
            ("Create(uint256,address,bytes32,string,ITerminal,address)",            SignatureKind::Event),
            ("SetHandle(uint256,bytes32,address)",                                  SignatureKind::Event),
            ("SetUri(uint256,string,address)",                                      SignatureKind::Event),
            ("TransferHandle(uint256,address,bytes32,bytes32,address)",             SignatureKind::Event),
            ("ClaimHandle(address,uint256,bytes32,address)",                        SignatureKind::Event),
            ("ChallengeHandle(bytes32,uint256,address)",                            SignatureKind::Event),
            ("RenewHandle(bytes32,uint256,address)",                                SignatureKind::Event),
            ("count()",                                                             SignatureKind::Function),
            ("uriOf(uint256)",                                                      SignatureKind::Function),
            ("handleOf(uint256)",                                                   SignatureKind::Function),
            ("projectFor(bytes32)",                                                 SignatureKind::Function),
            ("transferAddressFor(bytes32)",                                         SignatureKind::Function),
            ("challengeExpiryOf(bytes32)",                                          SignatureKind::Function),
            ("exists(uint256)",                                                     SignatureKind::Function),
            ("create(address,bytes32,string,ITerminal)",                            SignatureKind::Function),
            ("setHandle(uint256,bytes32)",                                          SignatureKind::Function),
            ("setUri(uint256,string)",                                              SignatureKind::Function),
            ("transferHandle(uint256,address,bytes32)",                             SignatureKind::Function),
            ("claimHandle(bytes32,address,uint256)",                                SignatureKind::Function),
            ("terminalDirectory()",                                                 SignatureKind::Function),
            ("migrationIsAllowed(ITerminal)",                                       SignatureKind::Function),
            ("pay(uint256,address,string,bool)",                                    SignatureKind::Function),
            ("addToBalance(uint256)",                                               SignatureKind::Function),
            ("allowMigration(ITerminal)",                                           SignatureKind::Function),
            ("migrate(uint256,ITerminal)",                                          SignatureKind::Function),
            ("DeployAddress(uint256,string,address)",                               SignatureKind::Event),
            ("SetTerminal(uint256,ITerminal,address)",                              SignatureKind::Event),
            ("SetPayerPreferences(address,address,bool)",                           SignatureKind::Event),
            ("projects()",                                                          SignatureKind::Function),
            ("terminalOf(uint256)",                                                 SignatureKind::Function),
            ("beneficiaryOf(address)",                                              SignatureKind::Function),
            ("unstakedTicketsPreferenceOf(address)",                                SignatureKind::Function),
            ("addressesOf(uint256)",                                                SignatureKind::Function),
            ("deployAddress(uint256,string)",                                       SignatureKind::Function),
            ("setTerminal(uint256,ITerminal)",                                      SignatureKind::Function),
            ("setPayerPreferences(address,bool)",                                   SignatureKind::Function),
        ];

        let actual_signatures = parser::from_sol(
            &std::fs::read_to_string("../res/sol/0x8BC61d005443F764D1D0d753F6Ec6f9B7eAe33B4").unwrap(),
        );

        assert_eq!(actual_signatures.len(), expected_signatures.len());
        for (i, actual_signature) in actual_signatures.iter().enumerate() {
            assert_eq!(actual_signature.text, expected_signatures[i].0);
            assert_eq!(actual_signature.kind, expected_signatures[i].1);
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

        assert_eq!(signatures[1].text, "Transfer(address,address,uint256)");
        assert_eq!(signatures[1].kind, SignatureKind::Event);

        assert_eq!(signatures[2].text, "Recv(address,address,uint256)");
        assert_eq!(signatures[2].kind, SignatureKind::Error);

        assert_eq!(signatures[3].text, "safeTransferFrom(address,address,uint256)");
        assert_eq!(signatures[3].kind, SignatureKind::Function);

        assert_eq!(signatures[4].text, "toHexString(uint256,uint256)");
        assert_eq!(signatures[4].kind, SignatureKind::Function);

        assert_eq!(signatures[5].text, "functionCall(address,bytes,string)");
        assert_eq!(signatures[5].kind, SignatureKind::Function);

        assert_eq!(signatures[6].text, "_transfer(address,address,uint256)");
        assert_eq!(signatures[6].kind, SignatureKind::Function);

        assert_eq!(signatures[7].text, "tokenURI(uint256)");
        assert_eq!(signatures[7].kind, SignatureKind::Function);

        assert_eq!(signatures[8].text, "doesntWorkButNowDoesBecauseItsFixedYay(address,uint256)");
        assert_eq!(signatures[8].kind, SignatureKind::Function);
    }
}
