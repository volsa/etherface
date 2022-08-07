import React from 'react'

const SearchBar = ({ input, placeholder, submitHandler, changeHandler }) => {
    return (
        <form onSubmit={submitHandler}>
            <input
                type='text'
                className='p-1 w-full text-center outline-none rounded bg-gray-100'
                placeholder={placeholder}
                value={input}
                onChange={changeHandler}
            />
        </form>
    )
}

export default SearchBar