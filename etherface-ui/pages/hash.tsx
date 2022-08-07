import axios from 'axios'
import React, { useState } from 'react'
import Alert from '../components/Alert'
import Navbar from '../components/Navbar'
import SearchBar from '../components/SearchBar'
import Table from '../components/Table'
import { Response, Signature } from '../lib/types'
import Head from 'next/head'



const Hash = () => {
    const [input, setInput] = useState('')
    const [query, setQuery] = useState('')
    const [errorCode, setErrorCode] = useState<number | null>()
    const [hashCollision, setHashCollision] = useState(false)
    const [signature, setSignature] = useState<Signature | null>()
    const [showSources, setShowSources] = useState('Github')

    const columnsSignature = [
        {
            header: 'Text',
            accessor: 'text',
            clickAction: () => undefined,
        },
        {
            header: 'Hash',
            accessor: 'hash',
            style: 'underline underline-offset-4 duration-200 cursor-pointer hover:text-black',
            clickAction: (event) => { setInput(event); setQuery(event); },
        }
    ]

    const columnsGithub = [
        {
            header: 'Name',
            accessor: 'name',
            accessorUrl: 'html_url',
            style: 'underline underline-offset-4 duration-200 cursor-pointer hover:text-black',
            clickAction: () => undefined,
        },
        {
            header: 'Stars',
            accessor: 'stargazers_count',
            clickAction: () => undefined,
        },
        {
            header: 'Solidity Ratio',
            accessor: 'solidity_ratio',
            clickAction: () => undefined,
        },
        {
            header: 'Created At',
            accessor: 'created_at',
            isDate: true,
            clickAction: () => undefined,
        },
        {
            header: 'Updated At',
            accessor: 'updated_at',
            isDate: true,
            clickAction: () => undefined,
        }
    ]

    const columnsEtherscan = [
        {
            header: 'Contract Name',
            accessor: 'name',
            accessorUrl: 'url',
            style: 'underline underline-offset-4 duration-200 cursor-pointer hover:text-black',
            clickAction: () => undefined,
        },
        {
            header: 'Compiler Version',
            accessor: 'compiler_version',
            clickAction: () => undefined,
        },
        {
            header: 'Added At',
            accessor: 'added_at',
            isDate: true,
            clickAction: () => undefined,
        }
    ]

    const submitHandler = (event) => {
        event.preventDefault()
        setQuery(input)
        setSignature(null);
    }

    const changeHandler = (event) => {
        setInput(event.target.value)
    }

    const fetcherHashEndpoint = async (query: string, page: number) => {
        setHashCollision(false);
        setErrorCode(null);

        let response = await axios.get(
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/signatures/hash/${query}/${page}`, {
            validateStatus: null // https://axios-http.com/docs/req_config
        }
        );

        let data = response.data;
        let isError = response.status != 200 ? true : false;
        let statusCode = response.status;

        if (!isError) {
            if (data.total_items == 1) {
                setSignature(data.items[0]);
                console.log(data.items[0].id)
            }

            if (data.total_items > 1) {
                setHashCollision(true)
            }
        }

        if (isError) {
            setErrorCode(statusCode);
        }

        return {
            data,
            isError,
            statusCode,
        }
    }

    const fetcherGithubEndpoint = async (query: string, page: number) => {
        setErrorCode(null);
        let response = await axios.get(
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/sources/github/${query}/${page}`, {
            validateStatus: null // https://axios-http.com/docs/req_config
        }
        );

        let data = response.data;
        let isError = response.status != 200 ? true : false;
        let statusCode = response.status;

        if (isError) {
            setErrorCode(statusCode);
        }

        return {
            data,
            isError,
            statusCode,
        }
    }

    const fetcherEtherscanEndpoint = async (query: string, page: number) => {
        setErrorCode(null);
        let response = await axios.get(
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/sources/etherscan/${query}/${page}`, {
            validateStatus: null // https://axios-http.com/docs/req_config
        }
        );

        let data = response.data;
        let isError = response.status != 200 ? true : false;
        let statusCode = response.status;

        if (isError) {
            setErrorCode(statusCode);
        }

        return {
            data,
            isError,
            statusCode,
        }
    }

    return (
        <div>
            <Navbar />
            <Head><title>Ethereum Signature Database</title></Head>
            <div className='grid grid-cols-8'>
                <div className='col-start-3 col-end-7'>
                    <div className='col-start-2 col-end-8 space-y-2 mt-4'>
                        <div className='space-y-2'>
                            <SearchBar
                                input={input}
                                placeholder={'Find signatures by their hash, e.g. 70a08231 (or the full hash)'}
                                submitHandler={submitHandler}
                                changeHandler={changeHandler}
                            />
                            {!query && <Alert
                                kind='info'
                                infoMessage={
                                    <div className='text-left flex flex-col gap-y-2'>
                                        <span className='text-xl text-center'>What&apos;s this?</span>
                                        <span>
                                            From <a href='https://www.4byte.directory/' className='underline underline-offset-auto'>4Byte</a>: &quot;Function calls in the Ethereum Virtual Machine are specified by the first four bytes of data sent with a transaction. These 4-byte signatures are defined as the first four bytes of the Keccak hash (SHA3) of the canonical representation of the function signature. The database also contains mappings for event signatures. Unlike function signatures, they are defined as 32-bytes of the Keccak hash (SHA3) of the canonical form of the event signature. Since this is a one-way operation, it is not possible to derive the human-readable representation of the function or event from the bytes signature. This database is meant to allow mapping those bytes signatures back to their human-readable versions.&quot;
                                            Besides providing such bytes signatures, this database slightly differs from 4Byte in that we automatically scrape and store content from GitHub, Etherscan and 4Byte allowing us find signatures with their potential source code.
                                        </span>
                                    </div>
                                }
                                errorMessage={undefined}
                                statusCode={undefined}
                                query={undefined}
                            />}
                        </div>
                        {signature &&
                            <div className='text-center font-mono'>
                                {signature.text}
                            </div>
                        }
                    </div>

                    <div className='mt-2'>
                        {hashCollision && <Alert
                            kind='info'
                            infoMessage={'Found multiple signatures, to see their sources pick one by clicking on their hash'}
                            errorMessage={undefined}
                            statusCode={null}
                            query={null}
                        />}

                        {errorCode && !signature && <Alert
                            kind='error'
                            infoMessage={undefined}
                            errorMessage={undefined}
                            statusCode={errorCode}
                            query={query}
                        />}

                        {errorCode && signature && <Alert
                            kind='error'
                            infoMessage={undefined}
                            errorMessage={`The given signature was not found on ${showSources}`}
                            statusCode={undefined}
                            query={undefined}
                        />}
                    </div>
                </div>

                <div className='col-start-2 col-end-8'>
                    <div className='flex justify-end'>
                        {signature &&
                            <button className='border rounded px-2'
                                onClick={() => { showSources === 'Github' ? setShowSources('Etherscan') : setShowSources('Github') }}>
                                {showSources === 'Github' ? 'Show Etherscan sources' : 'Show Github sources'}
                            </button>
                        }
                    </div>

                    {!signature && query.length != 0 && <Table
                        fetcher={fetcherHashEndpoint}
                        showPagination={false}
                        query={query}
                        queryKind={undefined}
                        columns={columnsSignature} />
                    }

                    {signature && showSources === 'Github' && <Table
                        fetcher={fetcherGithubEndpoint}
                        query={signature.id}
                        queryKind={undefined}
                        columns={columnsGithub} />
                    }

                    {signature && showSources === 'Etherscan' && <Table
                        fetcher={fetcherEtherscanEndpoint}
                        query={signature.id}
                        queryKind={undefined}
                        columns={columnsEtherscan} />
                    }
                </div>
            </div>
        </div >
    )
}

export default Hash