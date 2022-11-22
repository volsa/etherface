import Layout from "../components/Layout"
import LinkItem from "../components/LinkItem"
import Navbar from "../components/Navbar"
import Paragraph from "../components/Paragraph"

const About = () => {
    return (
        <Layout>
            <Navbar />

            <div className="flex flex-col items-center">
                <Paragraph
                    title='What is this?'
                    content={
                        <div>
                            Every transaction in Ethereum can carry additional input data, for example take the
                            following arbitrary transaction found on <LinkItem text='Etherscan' url='https://etherscan.io/tx/0x2b930225479934eda949c3c2b0f3af5d5fd60136f7c9f0d5bbabf569def1f8a8' />.
                            Its input data consists of <pre>0x095ea7b30000000000000000000000008bc3702c35d33e5df7cb0f06cb72a0c34ae0c56f00000000000000000000000000000000000000000000000ee5c13efe85190000</pre>
                            and at first sight might look uninteresting. This specific piece of data however is encoded <LinkItem text='Keccak256' url='https://emn178.github.io/online-tools/keccak_256.html' /> hash
                            and is essential when communicating with smart contracts. Specifically the first 4 byte of this input data, i.e. <code>0x095ea7b3</code>, specify which function in the
                            smart contract gets executed. What exactly does <code>0x095ea7b3</code> translate to though? Clearly some mapping between hashes and their clear form is needed to decode these functions.
                            Luckily databases full of such mappings exist and you are currently on one of them. Etherfaces database for example has a mapping between <code>0x095ea7b3</code> and <code>approve(address,uint256)</code>.
                            Not only do we now know there is a transaction between A and B but more importantly which function is executed, making such contract transaction more transparent.
                            <br />
                            <br />
                            Such signature databases are nothing new, in fact there were two other databases prior to Etherface namely <LinkItem text='4Byte' url='https://www.4byte.directory/' /> and <LinkItem text='samczsun' url='https://sig.eth.samczsun.com/' />.
                            While these two databases exceed in their tasks, they lack two important features: Self-Sustainability and Source-Code References. Specifically the 4Byte and samczsun databases are populated by user-input with no references where these signatures 
                            came from. Etherface solves both these issues by 24/7 monitoring and crawling 4Byte, Etherscan and GitHub to automatically find signatures while also leaving references where these signatures were found at. 
                            With over 2.180.000 signatures Etherface currently hosts more signatures than 4Byte or samczun, proving that 24/7 monitoring and crawling proved to be the right step. 
                            This is no competition however, as all these signature databases try to give a useful tool back to the Ethereum community ♡.
                        </div>
                    }
                />

                <Paragraph
                    title='Cite Etherface'
                    content={
                        <div>
                            <pre>
                                {`
@misc{Etherface,
    author={},
    title={},
    year={},
    url={},
}
                            `}
                            </pre>
                        </div>
                    }
                />
            </div>

        </Layout>
    )
}

export default About
