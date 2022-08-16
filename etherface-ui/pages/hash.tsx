import axios from 'axios'
import React, { useState } from 'react'
import Alert from '../components/Alert'
import Layout from '../components/Layout'
import Navbar from '../components/Navbar'
import SearchBar from '../components/SearchBar'
import Table from '../components/Table'
import { Signature, SignatureKind } from '../lib/types'



const Hash = () => {
    const [input, setInput] = useState('')
    const [query, setQuery] = useState('')
    const [queryKind, setQueryKind] = useState<SignatureKind | null>()
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

        if (input.startsWith('f#')) {
            setQuery(input.slice(2))
            setQueryKind(SignatureKind.Function)
            setSignature(null);
            return;
        }

        if (input.startsWith('e#')) {
            setQuery(input.slice(2))
            setQueryKind(SignatureKind.Event)
            setSignature(null);
            return;
        }

        if (input.startsWith('err#')) {
            setQuery(input.slice(4))
            setQueryKind(SignatureKind.Error)
            setSignature(null);
            return;
        }

        setQuery(input)
        setQueryKind(SignatureKind.All)
        setSignature(null);
    }

    const changeHandler = (event) => {
        setInput(event.target.value.trim())
    }

    const fetcherHashEndpoint = async (query: string, page: number) => {
        setHashCollision(false);
        setErrorCode(null);

        let response = await axios.get(
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/signatures/hash/${queryKind}/${query}/${page}`, {
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
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/sources/github/${queryKind}/${query}/${page}`, {
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
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/sources/etherscan/${queryKind}/${query}/${page}`, {
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
        <Layout>
            <Navbar />
            <div className='grid grid-cols-8'>
                <div className='col-start-3 col-end-7'>
                    <div className='col-start-2 col-end-8 space-y-2 mt-4'>
                        <div className='space-y-2'>
                            <SearchBar
                                input={input}
                                placeholder={'Find signatures by their hash, e.g. 0x70a08231 or the full hash'}
                                submitHandler={submitHandler}
                                changeHandler={changeHandler}
                            />
                            {!query && <Alert
                                kind='info'
                                infoMessage={
                                    <div className='flex flex-col text-left'>
                                        <span>Some tips:</span>
                                        <span>- Searches are case sensitive yielding signatures starting with your query</span>
                                        <span>- Filtering by function, event and error signatures is supported by starting your query with</span>
                                        <span className='ml-4'>- <code>f#</code> to find functions only, e.g. <code>f#0x70a08231</code></span>
                                        <span className='ml-4'>- <code>e#</code> to find events only, e.g. <code>e#0x2d339b1e</code></span>
                                        <span className='ml-4'>- <code>err#</code> to find errors only, e.g. <code>err#0xfe7507e6</code></span>
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
        </Layout>
    )
}

export default Hash