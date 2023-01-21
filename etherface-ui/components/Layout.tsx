import Head from 'next/head'
import React from 'react'

const Layout = ({ children }) => {
    return (
        <div>
            <Head>
                <title>Ethereum Signature Database</title>
                <meta name="description" content="Search for Ethereum function, event and error signatures from over 2 million unique signatures."/>
            </Head>
            <div>
                {children}
            </div>
        </div>
    )
}

export default Layout