import Link from 'next/link'
import { useRouter } from 'next/router'
import React from 'react'
import NavbarItem from './NavbarItem'


const Navbar = () => {
    const route = useRouter().pathname;

    return (
        <div className='flex flex-col items-center'>
            <div className='w-1/2 mx-2 my-2'>
                <div className='flex justify-between'>
                    <div className='space-x-2'>
                        <NavbarItem url='/statistics' value='Statistics' route={route} />
                        <NavbarItem url='/text' value='Text Search' route={route} />
                        <NavbarItem url='/hash' value='Hash Search' route={route} />
                    </div>

                    <div className='space-x-2'>
                        <NavbarItem url='/about' value='About' route={route} />
                        <NavbarItem url='https://github.com/volsa/etherface' value='Contribute️' route={route} />
                        <NavbarItem url='/api-documentation' value='API Documentation' route={route} />
                    </div>
                </div>
            </div>
        </div>
    )
}

export default Navbar