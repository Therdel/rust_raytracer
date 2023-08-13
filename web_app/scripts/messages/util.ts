export async function fetch_into_array(url: string): Promise<Uint8Array> {
    let response: Response = await fetch(url)
    if (!response.ok) {
        throw new Error(`Error fetching ${url}: ${response.status} ${response.statusText}`)
    }
    let array_buffer = await (response.arrayBuffer())
    return new Uint8Array(array_buffer)
}

export async function fetch_into_shared_buffer(url: string): Promise<SharedArrayBuffer> {
    let buffer_u8 = await fetch_into_array(url)
    let buffer_shared = new SharedArrayBuffer(buffer_u8.byteLength)
    new Uint8Array(buffer_shared).set(buffer_u8)

    return buffer_shared
}

export function isAbsoluteURL(url: string): boolean {
    return URL.canParse(url)
}