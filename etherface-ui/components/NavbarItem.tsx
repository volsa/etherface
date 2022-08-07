import Link from 'next/link'
import React from 'react'

const NavbarItem = ({ url, value, route }) => {
    return (
        <Link href={url}>
            <a className={`p-2 hover:text-black duration-200 ${route == url ? 'text-black' : 'text-gray-400'}`}
                target={url.startsWith('https://github.com/volsa') ? '_target' : '_self'}>
                {value}
            </a>
        </Link>
    )
}

export default NavbarItem