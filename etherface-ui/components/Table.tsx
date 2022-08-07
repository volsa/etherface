import React, { useEffect, useRef, useState } from 'react'
import Alert from './Alert'

const Table = ({ query, queryKind, showPagination = true, fetcher, columns }) => {
    const [items, setItems] = useState<[]>()
    const [totalPages, setTotalPages] = useState(-1)
    const [totalItems, setTotalItems] = useState(-1)
    const [currentPage, setCurrentPage] = useState(1)

    const [isFetching, setIsFetching] = useState(true)
    const [canGetNextPage, setCanGetNextPage] = useState(true)
    const [canGetPreviousPage, setCanGetPreviousPage] = useState(false)

    const [prevQuery, setPrevQuery] = useState(query)

    const initialRender = useRef(true)
    // const [debug, setDebug] = useState(true)
    const [debug, setDebug] = useState(false)

    const fetchData = async () => {
        setIsFetching(true)
        const { data, isError, statusCode } = await fetcher(query, currentPage)
        setIsFetching(false)

        setItems(data.items)
        setTotalItems(data.total_items)
        setTotalPages(data.total_pages)

        currentPage === data.total_pages ? setCanGetNextPage(false) : setCanGetNextPage(true)
        currentPage === 1 ? setCanGetPreviousPage(false) : setCanGetPreviousPage(true)
    }

    useEffect(() => {
        if (query != prevQuery) {
            setCurrentPage(1);
            setPrevQuery(query)
        }

        fetchData()
    }, [query, queryKind, currentPage])

    return (
        <div>
            {!items && isFetching && <div className='text-center mt-2'>Fetching data...</div>}
            {items &&
                <div>
                    {debug && <pre>{JSON.stringify({ totalPages, totalItems, currentPage, isFetching, canGetNextPage, canGetPreviousPage, query, prevQuery }, null, 2)}</pre>}
                    <div className="mt-4 overflow-x-auto relative border rounded">
                        <table className="w-full text-sm text-left text-gray-500">
                            <thead className="text-xs text-gray-700 uppercase bg-gray-50">
                                <tr>
                                    {columns.map((column) => (
                                        <th className='py-2 px-4'
                                            key={column.header}
                                        >
                                            {column.header}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {items && items.map((item: any) => (
                                    <tr key={item['id']} className="bg-white border-b hover:bg-gray-50">
                                        {columns.map((column: any) => (
                                            <td className={`py-2 px-4 ${column.style}`}
                                                onClick={() => column.clickAction(item[column.accessor])}
                                                key={item[column.accessor]}
                                            >
                                                <a href={item[column.accessorUrl]} target='_blank' rel='noreferrer'> {column.isDate ? new Date(item[column.accessor]).toDateString() : item[column.accessor]} </a>
                                            </td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                    {showPagination &&
                        <div className='flex justify-between mt-2 mb-16'>
                            <div className='space-x-2'>
                                <button className='rounded border px-2' disabled={isFetching || !canGetPreviousPage} onClick={() => { setCurrentPage(1); window.scrollTo(0, 0) }} >{`<< First`}</button>
                                <button className='rounded border px-2' disabled={isFetching || !canGetPreviousPage} onClick={() => { setCurrentPage(currentPage - 1); window.scrollTo(0, 0) }} >{`< Prev`}</button>
                            </div>

                            <div>
                                {isFetching
                                    ? 'Fetching data...'
                                    // : `Page  ${currentPage} of ${totalPages}`
                                    :
                                    <>
                                        <label htmlFor='goto-page'>Page </label>
                                        <select name='goto-page' value={currentPage} onChange={(event) => { setCurrentPage(Number(event.target.value)); window.scrollTo(0, 0) }}>
                                            {[...Array(totalPages)].map((_, index) => (
                                                <option key={index}>{index + 1}</option>
                                            ))}
                                        </select>
                                    </>

                                }
                                {!isFetching && <span className='whitespace-pre-wrap'> of {totalPages}</span>}
                            </div>
                            <div className='space-x-2'>
                                <button className='rounded border px-2' disabled={isFetching || !canGetNextPage} onClick={() => { setCurrentPage(currentPage + 1); window.scrollTo(0, 0) }} >{`Next >`}</button>
                                <button className='rounded border px-2' disabled={isFetching || !canGetNextPage} onClick={() => { setCurrentPage(totalPages); window.scrollTo(0, 0) }} >{`Last >>`}</button>
                            </div>
                        </div>
                    }
                </div >
            }
        </div>
    )
}

export default Table