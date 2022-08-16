import axios from 'axios'
import React, { useState } from 'react'
import Alert from '../components/Alert'
import Layout from '../components/Layout'
import Navbar from '../components/Navbar'
import SearchBar from '../components/SearchBar'
import Table from '../components/Table'
import { SignatureKind } from '../lib/types'


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
        setInput(event.target.value.trim())
    }

    const fetcherTextEndpoint = async (query: string, page: number) => {
        setErrorCode(null)

        let response = await axios.get(
            `${process.env.ETHERFACE_REST_ADDRESS}/v1/signatures/text/${queryKind}/${query}/${page}`, {
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
        <Layout>
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
                                    <span>Some tips:</span>
                                    <span>- Searches are case sensitive yielding signatures starting with your query</span>
                                    <span>- To narrow down your search to signature names only, end your query with <code>(</code>, e.g. <code>balanceOf(</code></span>
                                    <span>- Filtering by function, event and error signatures is supported by starting your query with</span>
                                    <span className='ml-4'>- <code>f#</code> to find functions only, e.g. <code>f#balanceOf</code></span>
                                    <span className='ml-4'>- <code>e#</code> to find events only, e.g. <code>e#Transfer</code></span>
                                    <span className='ml-4'>- <code>err#</code> to find errors only, e.g. <code>err#Not</code></span>
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
        </Layout>
    )
}

export default Text