import React from 'react'
import Layout from '../components/Layout'
import Navbar from '../components/Navbar'

const LinkItem = ({ text, url }) => {
    return (<a target='_blank' rel="noreferrer" className='underline underline-offset-2 text-gray-400 hover:text-black duration-200' href={url}>{text}</a>)
}

const Paragraph = ({ title, content }) => {
    return (
        <div className='w-11/12 2xl:w-1/2 mt-4'>
            <div className='p-4'>
                <div className='text-2xl text-black mb-1'>{title}</div>
                <div className='text text-gray-600'>{content}</div>
            </div>
        </div>
    )
}

const ApiDocumentation = () => {
    return (
        <Layout>
                <Navbar />

                <div className='flex flex-col items-center'>
                    <Paragraph
                        title='Introduction'
                        content={
                            <div>
                                <ul className='list-disc list-inside'>
                                    <li className='list-item'>All listed API endpoints are paginated, returning 100 items per page starting at page 1</li>
                                    <li className='list-item'>Successful responses have the following JSON structure: <code className='text-sm'>{`{"total_pages": ..., "total_items": ..., "items": [ {...}, ...] }`}</code></li>
                                    <li className='list-item'>Unsuccessful responses either return the <code>400</code> or <code>404</code> HTTP status code</li>
                                    <li className='list-item'>Additionally the <code>429</code> status code might be added to the list depending on whether or not ratelimiting is neccessary (hopefully not, fingers crossed)</li>
                                    <li className='list-item'>Lastly, if you enjoy this project make sure to also star it on <LinkItem text='GitHub' url='https://github.com/volsa/etherface' /> {`<3`}</li>
                                </ul>
                            </div>
                        }
                    />

                    <Paragraph
                        title={<code>{`/v1/signatures/text/{kind}/{query}/{page}`}</code>}
                        content={
                            <div>
                                <p>Returns a paginated list of signatures where</p>
                                <ul className='list-disc list-inside'>
                                    <li className='list-item'><code>kind</code> is either <code>function</code>, <code>event</code>, <code>error</code> or <code>all</code></li>
                                    <li className='list-item'><code>query</code> is the signatures starting text representation (at least 3 characters long)</li>
                                    <li className='list-item'><code>page</code> is the page index, starting at 1</li>
                                </ul>
                                <p><b>Example:</b> <LinkItem text='api.etherface.io/v1/signatures/text/all/balanceOf/1' url='https://api.etherface.io/v1/signatures/text/all/balanceOf/1' /> returns all signatures starting with <code>balanceOf</code> (case sensitive!)</p>
                            </div>
                        }
                    />

                    <Paragraph
                        title={<code>{`/v1/signatures/hash/{kind}/{query}/{page}`}</code>}
                        content={
                            <div>
                                <p>Returns a paginated list of signatures where</p>
                                <ul className='list-disc list-inside'>
                                    <li className='list-item'><code>kind</code> is either <code>function</code>, <code>event</code>, <code>error</code> or <code>all</code></li>
                                    <li className='list-item'><code>query</code> is the signature hash (either 8 or 64 characters long, excluding the <code>0x</code> head which is also optional)</li>
                                    <li className='list-item'><code>page</code> is the page index, starting at 1</li>
                                </ul>
                                <p><b>Example:</b> <LinkItem text='api.etherface.io/v1/signatures/hash/all/70a08231/1' url='https://api.etherface.io/v1/signatures/hash/all/70a08231/1' /> returns all signatures starting with the hash <code>70a08231</code> (case sensitive!)</p>
                            </div>
                        }
                    />

                    <Paragraph
                        title={<code>{`/v1/sources/github/{kind}/{id}/{page}`}</code>}
                        content={
                            <div>
                                <p>Returns a paginated list of GitHub repositories ordered by their stargazers where </p>
                                <ul className='list-disc list-inside'>
                                    <li className='list-item'><code>kind</code> is either <code>function</code>, <code>event</code>, <code>error</code> or <code>all</code></li>
                                    <li className='list-item'><code>id</code> is the signatures internal ID, obtained by the <code>{`/v1/signatures/{text,hash}`}</code> endpoints</li>
                                    <li className='list-item'><code>page</code> is the page index, starting at 1</li>
                                </ul>
                                <p><b>Example:</b> <LinkItem text='api.etherface.io/v1/sources/github/all/7/1' url='https://api.etherface.io/v1/sources/github/all/7/1' /> returns all GitHub repositories which contain the <code>balanceOf(address)</code> signature / <code>70a08231</code> hash</p>
                            </div>
                        }
                    />

                    <Paragraph
                        title={<code>{`/v1/sources/etherscan/{kind}/{id}/{page}`}</code>}
                        content={
                            <div>
                                <p>Returns a paginated list of Etherscan addresses ordered by their added date where </p>
                                <ul className='list-disc list-inside'>
                                    <li className='list-item'><code>kind</code> is either <code>function</code>, <code>event</code>, <code>error</code> or <code>all</code></li>
                                    <li className='list-item'><code>id</code> is the signatures internal ID, obtained by the <code>{`/v1/signatures/{text,hash}`}</code> endpoints</li>
                                    <li className='list-item'><code>page</code> is the page index, starting at 1</li>
                                </ul>
                                <p><b>Example:</b> <LinkItem text='api.etherface.io/v1/sources/etherscan/all/7/1' url='https://api.etherface.io/v1/sources/etherscan/all/7/1' /> returns all Etherscan addresses which contain the <code>balanceOf(address)</code> signature / <code>70a08231</code> hash</p>
                            </div>
                        }
                    />
                </div>
        </Layout>
    )
}

export default ApiDocumentation