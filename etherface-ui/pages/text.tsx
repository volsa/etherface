import axios from 'axios'
import React, { useState } from 'react'
import Alert from '../components/Alert'
import Navbar from '../components/Navbar'
import SearchBar from '../components/SearchBar'
import Table from '../components/Table'
import { SignatureKind } from '../lib/types'
import Head from 'next/head'


const Text = () => {
    const [input, setInput] = useState('')
    const [query, setQuery] = useState('')
    const [queryKind, setQueryKind] = useState<SignatureKind | null>()
    const [errorCode, setErrorCode] = useState<number | null>()

    const submitHandler = (event) => {
        event.preventDefault()

        if (input.startsWith('f#')) {
            setQuery(input.slice(2))
            setQueryKind(SignatureKind.Function)
            return;
        }

        if (input.startsWith('e#')) {
            setQuery(input.slice(2))
            setQueryKind(SignatureKind.Event)
            return;
        }

        if (input.startsWith('err#')) {
            setQuery(input.slice(4))
            setQueryKind(SignatureKind.Error)
            return;
        }

        setQuery(input)
        setQueryKind(SignatureKind.All)
    }

    const changeHandler = (event) => {
        setInput(event.target.value)
    }

    const fetcherTextEndpoint = async (query: string, page: number) => {
        setErrorCode(null)

        let response = await axios.get(
            `${process.env.REST_ADDR}/v1/signatures/text/${queryKind}/${query}/${page}`, {
            validateStatus: null // https://axios-http.com/docs/req_config
        }
        );

        let data = response.data;
        let isError = response.status != 200 ? true : false;
        let statusCode = response.status;

        if (isError) {
            setErrorCode(statusCode)
        }

        return {
            data,
            isError,
            statusCode,
        }
    }


    const columnDef = [
        {
            header: 'Text',
            accessor: 'text',
            clickAction: () => undefined,
        },
        {
            header: 'Hash',
            accessor: 'hash',
            clickAction: () => undefined,
        }
    ]

    return (
        <div>
            <Head><title>Ethereum Signature Database</title></Head>
            <Navbar />
            <div className='grid grid-cols-8'>
                <div className='col-start-3 col-end-7'>
                    <div className='pt-4 space-y-2'>
                        <SearchBar
                            input={input}
                            placeholder='Find signatures by their text, e.g. balanceOf'
                            submitHandler={submitHandler}
                            changeHandler={changeHandler}
                        />
                        {!query && <Alert
                            kind='info'
                            infoMessage={
                                <div className='flex flex-col text-left'>
                                    <span>Some things to know:</span>
                                    <span>- This page currently <b>only</b> supports finding signatures found on GitHub</span>
                                    <span>- Searches are case sensitive yielding signatures starting with your query</span>
                                    <span>- Filtering by function, event or error signatures is supported by starting your query with <span className='font-mono'>f#</span>, <span className='font-mono'>e#</span> or <span className='font-mono'>#err</span> respectively, e.g.</span>
                                    <span className='ml-4'>- <span className='font-mono'>f#balanceOf</span>, <span className='font-mono'>e#Transfer</span>, <span className='font-mono'>err#Not</span></span>
                                </div>
                            }
                            errorMessage={undefined}
                            statusCode={undefined}
                            query={undefined}
                        />}
                    </div>

                    {errorCode &&
                        <div className='pt-2'>
                            <Alert
                                kind='error'
                                infoMessage={undefined}
                                errorMessage={undefined}
                                statusCode={errorCode}
                                query={query}
                            />
                        </div>
                    }

                </div>

                <div className='col-start-2 col-end-8'>
                    {query && <Table query={query} queryKind={queryKind} fetcher={fetcherTextEndpoint} columns={columnDef} />}
                </div>

            </div>
        </div>
    )
}

export default Text