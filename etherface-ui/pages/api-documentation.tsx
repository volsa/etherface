import React from 'react'
import Alert from '../components/Alert'
import Navbar from '../components/Navbar'
import Head from 'next/head'

const Paragraph = ({ title, content }) => {
    return (
        <div>
            <div className='text-3xl font-medium text-black text-left'>{title}</div>
            <div className='leading-6 text-gray-600 mt-1'>
                {content}
            </div>
        </div>
    )
}

const SpanMono = ({ content }) => {
    return (
        <span className='font-mono'>{content}</span>
    )
}

const ApiDocumentation = () => {
    return (
        <div>
            <Navbar />
            <Head><title>Ethereum Signature Database</title></Head>
            <div className='grid grid-cols-6'>
                <div className='col-start-3 col-span-2 mt-4'>
                    <Alert
                        kind='info'
                        infoMessage='The documentation is currently work in progress and will receive an overhaul soon-ish'
                        errorMessage={undefined}
                        statusCode={undefined}
                        query={undefined}
                    />

                    <div className='flex flex-col gap-y-12 mt-4'>
                        <Paragraph
                            title='Introduction'
                            content={
                                <>
                                    <ul className='list-disc list-inside'>
                                        <li><b>Ratelimit:</b> Currently no ratelimiting is enforced on any endpoint, this might however change in the future if either responses become too slow or the API is being abused.</li>
                                        <li><b>Status Codes:</b> Endpoints might return 200 (OK), 400 (Bad Request), 404 (Not Found) or 429 (Too Many Requests) HTTP status codes. Furthermore status codes such as 500 might also be returned implicitly, as such make sure to cover both expected and unexcepted status codes.</li>
                                        <li><b>Pagination:</b> Most endpoints return a paginated response with the following structure <span className='font-mono'>{`{"total_pages": ..., "total_items": ..., "items": [ {...}, {...}, ...] }`}</span></li>
                                    </ul>
                                </>
                            } />

                        <Paragraph
                            title={<span className='font-mono'>{`/v1/signatures/hash/{signature}/{page}`}</span>}
                            content={
                                <div className='whitespace-pre-line'>
                                    <p>Returns a paginated list of signatures where</p>
                                    <ul className='list-disc list-inside'>
                                        <li><b>signature:</b> is the function selector (hash), either 8 or 64 characters long</li>
                                        <li><b>page:</b> is the page index (starting at 1)</li>
                                    </ul>
                                    <p>Example: <a href='https://api.etherface.io/v1/signatures/hash/70a08231/1' className='underline underline-offset-2 hover:text-black duration-200'>https://api.etherface.io/v1/signatures/hash/70a08231/1</a> returns all signatures starting with <span className='font-mono'>70a08231</span> where the page index is 1.</p>
                                    <p className='mt-2'><b>Note:</b> Currently does not support filterting by kind. If you feel like this should be supported feel free to open a <a className='underline underline-offset-2 hover:text-black duration-200' href='https://github.com/volsa/etherface/issues/new'>GitHub issue</a>.</p>
                                </div>
                            } />

                        <Paragraph
                            title={<span className='font-mono'>{`/v1/signatures/text/{kind}/{signature}/{page}`}</span>}
                            content={
                                <div className='whitespace-pre-line'>
                                    <p>Returns a paginated list of signatures where</p>
                                    <ul className='list-disc list-inside'>
                                        <li><b>kind:</b> is the signature kind which must be <SpanMono content={'all'} />, <SpanMono content='function' />, <SpanMono content='event' /> or <SpanMono content='error' /></li>
                                        <li><b>signature:</b> is the starting text representation of the signature (must be at least 3 characters long)</li>
                                        <li><b>page:</b> is the page index (starting at 1)</li>
                                    </ul>
                                    <p>Example: <a href='https://api.etherface.io/v1/signatures/text/function/balanceOf/1' className='underline underline-offset-2 hover:text-black duration-200'>https://api.etherface.io/v1/signatures/text/function/balanceOf/1</a> returns all function signatures starting with <span className='font-mono'>balanceOf</span> where the page index is 1.</p>
                                    <p className='mt-2'><b>Note:</b> Currently only returns signatures found from GitHub, i.e. Etherscan and 4Byte signatures are excluded. If you feel like this should be supported feel free to open a <a className='underline underline-offset-2 hover:text-black duration-200' href='https://github.com/volsa/etherface/issues/new'>GitHub issue</a>.</p>
                                </div>
                            } />

                        <Paragraph
                            title={<span className='font-mono'>{`/v1/sources/{source}/{id}/{page}`}</span>}
                            content={
                                <div className='whitespace-pre-line'>
                                    <p>Returns a paginated list of potential source code belonging to a signature where</p>
                                    <ul className='list-disc list-inside'>
                                        <li><b>source:</b> is the platform where the signature was found at which must be either <SpanMono content='github' /> or <SpanMono content='etherscan' /></li>
                                        <li><b>id:</b> is the signatures ID which can be obtained using the <SpanMono content='/v1/signatures/{hash,text}' /> endpoint</li>
                                        <li><b>page:</b> is the page index (starting at 1)</li>
                                    </ul>
                                    <p>Example: <a href='https://api.etherface.io/v1/sources/github/39/1' className='underline underline-offset-2 hover:text-black duration-200'>https://api.etherface.io/v1/sources/github/39/1</a> returns all GitHub repositories where the signature <SpanMono content='balanceOf(address)' /> was scraped from with the page index is 1.</p>
                                </div>
                            } />
                    </div>
                </div>
            </div>
        </div>
    )
}

export default ApiDocumentation