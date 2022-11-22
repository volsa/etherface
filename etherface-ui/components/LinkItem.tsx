const LinkItem = ({ text, url }) => {
    return (<a target='_blank' rel="noreferrer" className='underline underline-offset-2 text-gray-400 hover:text-black duration-200' href={url}>{text}</a>)
}

export default LinkItem