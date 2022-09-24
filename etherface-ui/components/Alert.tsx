import React from 'react'

const Alert = ({ kind, infoMessage, errorMessage, statusCode, query }) => {
    return (
        <div className={`p-2 text-center rounded ${kind === 'info'
            ? 'bg-blue-50 text-blue-900'
            : 'bg-red-50 text-red-900'}`
        }>
            {(() => {
                if (kind == 'info') {
                    return infoMessage
                }

                if (kind == 'error') {
                    if (errorMessage) {
                        return errorMessage;
                    }

                    if (statusCode == 400) {
                        return 'Invalid query, should be at least 3 characters long (text search) or 8 / 64 (hash search, excluding 0x)'
                    }

                    if (statusCode == 404) {
                        return `"${query}" does not exist`
                    }

                    return 'Unexpected error, try again later'
                }

            })()}
        </div>
    )
}

export default Alert
