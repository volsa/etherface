const Paragraph = ({ title, content }) => {
    return (
        <div className='w-11/12 2xl:w-1/2 mt-4'>
            <div>
                <div className='text-2xl text-black mb-1'>{title}</div>
                <div className='text text-gray-600'>{content}</div>
            </div>
        </div>
    )
}

export default Paragraph