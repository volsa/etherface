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
                            following (random) transaction found on <LinkItem text='Etherscan' url='https://etherscan.io/tx/0x2b930225479934eda949c3c2b0f3af5d5fd60136f7c9f0d5bbabf569def1f8a8' />.
                            It's input data consists of <pre>0x095ea7b30000000000000000000000008bc3702c35d33e5df7cb0f06cb72a0c34ae0c56f00000000000000000000000000000000000000000000000ee5c13efe85190000</pre>
                            and at first sight might look uninteresting. This specific piece of data however is encoded Keccak256 hash and is very important when communicating with smart contracts.
                            Specifically the first 4 byte of this input data, i.e. <code>0x095ea7b3</code>, specify which function in the smart contract should be executed. But what exactly
                            does <code>0x095ea7b3</code> translate to? This is where signature databases (also known as rainbow tables) come into play, mappings between hashes and their unprocessed text form.
                            For example <code>0x095ea7b3</code> would map to <code>approve(address,uint256)</code>. Great instead of only knowing there's a transaction between A and B we now also know the 
                            given <code>approve</code> function is executed, hence making transactions more transparent.
                            <br/>
                            <br/>
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