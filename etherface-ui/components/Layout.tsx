import Head from 'next/head'
import React from 'react'

const Layout = ({ children }) => {
    return (
        <div>
            <Head>
                <title>Ethereum Signature Database</title>
                <meta name="description" content="Ethereum Signature Database to find function, event and error signatures in the Ethereum Network."/>
            </Head>
            <div>
                {children}
            </div>
        </div>
    )
}

export default Layout