import axios from 'axios';
import React, { useEffect, useState } from 'react'
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import Navbar from '../components/Navbar'
import Head from 'next/head'
import Alert from '../components/Alert';
import { Statistics } from '../lib/types'


const Statistics = () => {
    const [items, setItems] = useState<Statistics>()

    const fetch = async () => {
        let response = await axios.get<Statistics>(
            `${process.env.REST_ADDR}/v1/statistics`, {
            validateStatus: null // https://axios-http.com/docs/req_config
        }
        );

        let responseData = response.data;
        let isError = response.status != 200 ? true : false;
        let statusCode = response.status;

        setItems(responseData)
    }

    useEffect(() => {
        fetch()
    }, [])

    const Card = ({ title, stat }) => {
        return (
            <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden border transform transition-all mb-4 w-full">
                <div className="bg-white p-5">
                    <div className="text-center">
                        <h3 className="text-sm leading-6 font-medium text-gray-400">{title}</h3>
                        <p className="text-3xl font-bold text-black">{stat.toLocaleString().replaceAll(',', '.')}</p>
                    </div>
                </div>
            </div>
        )
    }

    return (
        <div>
            <Head><title>Ethereum Signature Database</title></Head>

            <Navbar />
            <div className='grid grid-cols-8 text-md'>
                {items &&
                    <div className='mt-4 col-start-3 col-end-7'>
                        <Alert
                            kind='info'
                            infoMessage='Note: The following statistics are updated only once every day'
                            errorMessage={undefined}
                            statusCode={undefined}
                            query={undefined}
                        />

                        <div className='mt-4 gap-x-8 gap-y-2 grid grid-cols-2 2xl:grid-cols-4'>
                            <Card title={<span># Signatures</span>} stat={items.statistics_various_signature_counts.signature_count} />
                            <Card title={<span># Signatures from <a rel='noreferrer' target='_blank' href='https://www.github.com/' className='underline underline-offset-2'>GitHub</a></span>} stat={items.statistics_various_signature_counts.signature_count_github} />
                            <Card title={<span># Signatures from <a rel='noreferrer' target='_blank' href='https://www.4byte.directory/' className='underline underline-offset-2'>4Byte</a></span>} stat={items.statistics_various_signature_counts.signature_count_fourbyte} />
                            <Card title={<span># Signatures from <a rel='noreferrer' target='_blank' href='https://www.etherscan.io/' className='underline underline-offset-2'>Etherscan</a></span>} stat={items.statistics_various_signature_counts.signature_count_etherscan} />

                            {items.statistics_signature_kind_distribution.map((item) => {
                                return (
                                    <Card key={item.kind} title={`# ${item.kind.charAt(0).toUpperCase() + item.kind.slice(1)} Signatures`} stat={item.count} />
                                )
                            })}
                            <Card title={<span>Last weeks avg. inserts / day</span>} stat={items.statistics_various_signature_counts.average_daily_signature_insert_rate_last_week.toLocaleString().replaceAll(',', '.')} />

                        </div>

                        <div className='w-full h-72 text-center mt-8'>
                            <ResponsiveContainer width="100%" height="80%">
                                <AreaChart data={items.statistics_signature_insert_rate} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
                                    <XAxis dataKey="date" />
                                    <YAxis stroke='#eee' dataKey="count" />
                                    <CartesianGrid strokeDasharray="5 5" stroke='#eee' />
                                    <Tooltip />
                                    <Area name="# Signatures inserted" type="monotone" dataKey="count" stroke="#8884d8" fill='#8884d8' />
                                </AreaChart>
                            </ResponsiveContainer>
                        </div>

                        <div className='text-center'>
                            <span>Most popular signatures found on GitHub</span>
                            <div className="mt-1 text-gray-500 overflow-x-auto relative border rounded mb-8">
                                <table className="w-full text-sm text-left text-gray-500">
                                    <thead className="text-xs text-gray-700 uppercase bg-gray-50">
                                        <tr>
                                            <th className='py-2 px-4'>Rank</th>
                                            <th className='py-2 px-4'>Signature</th>
                                            <th className='py-2 px-4'>Scrape Count (includes duplicates)</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {items.statistics_signatures_popular_on_github.map((item, idx) => (
                                            <tr key={item.text} className="bg-white border-b hover:bg-gray-50">
                                                <td className='py-2 px-4'>{idx + 1}</td>
                                                <td className='py-2 px-4'>{item.text}</td>
                                                <td className='py-2 px-4'>{item.count}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                }
            </div>
        </div >
    )
}

export default Statistics
