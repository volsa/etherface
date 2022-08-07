export type Response<T> = {
    total_pages: number,
    total_items: number,
    items: Array<T>
}

export type Signature = {
    id: number,
    text: string,
    hash: string,
}

export interface Statistics {
    statistics_various_signature_counts: StatisticsVariousSignatureCounts
    statistics_signature_insert_rate: StatisticsSignatureInsertRate[]
    statistics_signature_kind_distribution: StatisticsSignatureKindDistribution[]
    statistics_signatures_popular_on_github: StatisticsSignaturesPopularOnGithub[]
}

export interface StatisticsVariousSignatureCounts {
    signature_count: number
    signature_count_github: number
    signature_count_etherscan: number
    signature_count_fourbyte: number
    average_daily_signature_insert_rate_last_week: number
    average_daily_signature_insert_rate_week_before_last: number
}

export interface StatisticsSignatureInsertRate {
    date: string
    count: number
}

export interface StatisticsSignatureKindDistribution {
    kind: string
    count: number
}

export interface StatisticsSignaturesPopularOnGithub {
    text: string
    count: number
}

export enum SignatureKind {
    All = 'all',
    Function = 'function',
    Event = 'event',
    Error = 'error',
}