import Head from 'next/head'
import React from 'react'

const Layout = ({ children }) => {
    return (
        <div>
            <Head>
                <title>Ethereum Signature Database</title>
                <meta name="description" content="Just like 4Byte, Etherface is an Ethereum Signature Database for inspecting function, event and error signatures within the Ethereum network. Unlike 4Byte however, Etherface scrapes these signatures from multiple sites such as GitHub, Etherscan and 4Byte 24/7 yielding more than 1.8 million unique signatures."/>
            </Head>
            <div>
                {children}
            </div>
        </div>
    )
}

export default Layout