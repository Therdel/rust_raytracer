import { fetch_into_shared_buffer, isAbsoluteURL } from "./util"

export class AssetStore {
    public readonly BASE_PATHS = {
        models: "./res/models",
        scenes: "./res/scenes"
    }

    private assets = new Map<string, SharedArrayBuffer>()

    getAssetsMap(): Map<string, SharedArrayBuffer> {
        return this.assets
    }
    
    static fromMap(assets: Map<string, SharedArrayBuffer>): AssetStore {
        const asset_store = new AssetStore()
        asset_store.assets = assets
        return asset_store
    }

    has(key: string): boolean {
        return this.assets.has(key)
    }

    get(key: string): SharedArrayBuffer | undefined {
        return this.assets.get(key)
    }

    async putScene(key: string, source?: SharedArrayBuffer): Promise<SharedArrayBuffer> {
        if (this.assets.has(key)) {
            throw new Error(`Key ${key} already exists`)
        }

        const buffer = await this.put(key, source)
        await this.cacheSceneDependencies(buffer)
        return buffer
    }

    async put(key: string, source?: string | SharedArrayBuffer): Promise<SharedArrayBuffer> {
        let buffer: SharedArrayBuffer

        if (this.assets.has(key)) {
            return this.assets.get(key)!
        }

        const is_upload = source instanceof SharedArrayBuffer
        if (is_upload) {
            buffer = source
        } else {
            const url = source || this.resolveUrl(key)
            buffer = await fetch_into_shared_buffer(url)
        }

        this.assets.set(key, buffer)
        return buffer
    }
    
    private resolveUrl(key: string): string {
        if (isAbsoluteURL(key)) {
            return key
        }

        const isModel = /\.(gltf|glb|obj|fbx)$/i.test(key)
        console.debug(`Resolved key=${key} as ${isModel?'Model':'Scene'}`)

        const basePath = isModel ? this.BASE_PATHS.models : this.BASE_PATHS.scenes
        
        return `${basePath}/${key}`
    }

    private async cacheSceneDependencies(sceneBuffer: SharedArrayBuffer): Promise<void> {
        try {
            const buffer_nonshared = new Uint8Array(sceneBuffer.byteLength)
            buffer_nonshared.set(new Uint8Array(sceneBuffer))
            const sceneText = new TextDecoder().decode(buffer_nonshared)
            const scene = JSON.parse(sceneText)

            if (!scene.meshes?.length) return

            const cachePromises = scene.meshes
                .map((mesh: any) => mesh.file_name)
                .filter((fileName: string) => !this.has(fileName))
                .map((fileName: string) => this.put(fileName))

            await Promise.all(cachePromises)
        } catch (error: unknown) {
            console.error("Failed to parse scene dependencies:", error)
        }
    }
}