import Link from 'next/link'
import { useRouter } from 'next/router'
import React from 'react'
import NavbarItem from './NavbarItem'


const Navbar = () => {
    const route = useRouter().pathname;

    return (
        <div className='block py-1 border-b'>
            <div className='mx-2 my-2'>
                <div className='flex justify-between'>
                    <div className='justify-left space-x-2'>
                        <NavbarItem url='/statistics' value='Statistics' route={route} />
                        <NavbarItem url='/text' value='Text Search' route={route} />
                        <NavbarItem url='/hash' value='Hash Search' route={route} />
                    </div>

                    <div className='justify-end space-x-2'>
                        <NavbarItem url='/api-documentation' value='API Documentation' route={route} />
                        <NavbarItem url='https://github.com/volsa/etherface' value='Contribute ❤️' route={route} />
                    </div>
                </div>
            </div>
        </div>
    )
}

export default Navbar