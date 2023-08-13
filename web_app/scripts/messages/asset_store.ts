import { fetch_into_shared_buffer, isAbsoluteURL } from "./util.js"

export class AssetStore {
    public readonly MODELS_BASE_PATH = "./res/models"
    public readonly SCENES_BASE_PATH = "./res/scenes"

    private assets = new Map<string, SharedArrayBuffer>()

    // sets the current scene, then fetches and caches its json and referenced meshes as assets
    async put_scene_and_cache_assets(scene_url_or_filename: string): Promise<SharedArrayBuffer> {
        const absolute_url = this.absolute_scene_url(scene_url_or_filename)
        // const scene_file_buffer = await this.put(scene_url)
        const scene_file_buffer = await this.put(scene_url_or_filename, absolute_url)

        await this.cache_scene_meshes(scene_file_buffer)
        return scene_file_buffer
    }

    get_scene(scene_url_or_filename: string): SharedArrayBuffer | undefined {
        return this.get(scene_url_or_filename)
    }

    get_mesh(mesh_url_or_filename: string): SharedArrayBuffer | undefined {
        return this.get(mesh_url_or_filename)
    }

    iterate(): MapIterator<[string, SharedArrayBuffer]> {
        return this.assets.entries()
    }

    serialize(): Map<string, SharedArrayBuffer> {
        return this.assets
    }

    static deserialize(serialized_assets: Map<string, SharedArrayBuffer>) {
        const asset_store = new AssetStore()
        asset_store.assets = serialized_assets;
        return asset_store
    }

    private absolute_scene_url(scene_url_or_filename: string): string {
        const is_absolute: boolean = isAbsoluteURL(scene_url_or_filename)
        let absolute_url = scene_url_or_filename
        if (!is_absolute) {
            absolute_url = this.SCENES_BASE_PATH + '/' + scene_url_or_filename
        }

        return absolute_url
    }

    private absolute_mesh_url(mesh_url_or_filename: string): string {
        const is_absolute: boolean = isAbsoluteURL(mesh_url_or_filename)
        let absolute_url = mesh_url_or_filename
        if (!is_absolute) {
            absolute_url = this.MODELS_BASE_PATH + '/' + mesh_url_or_filename
        }

        return absolute_url
    }

    // parse the scene to cache its meshes
    private async cache_scene_meshes(scene_file_buffer: SharedArrayBuffer) {
        const scene_file_buffer_nonshared_for_decoding = new ArrayBuffer(scene_file_buffer.byteLength)
        const scene_file_buffer_u8 = new Uint8Array(scene_file_buffer_nonshared_for_decoding)
        scene_file_buffer_u8.set(new Uint8Array(scene_file_buffer))
        const buffer_u8 = new Uint8Array(scene_file_buffer_u8);
        const scene_str = new TextDecoder().decode(buffer_u8);
        const scene = JSON.parse(scene_str);

        // TODO validate scene, throw if of wrong format

        if ("meshes" in scene) {
            let promises: Promise<SharedArrayBuffer>[] = []
            for (const mesh of scene.meshes) {
                const mesh_url_or_filename: string = mesh.file_name
                const absolute_url = this.absolute_mesh_url(mesh_url_or_filename)

                if (this.contains(mesh_url_or_filename)) {
                    continue
                }

                console.debug(`Caching new mesh: name=${mesh_url_or_filename} absolute_url=${absolute_url}`)
                const promise = this.put(mesh_url_or_filename, absolute_url)
                promises.push(promise)
            }
            await Promise.all(promises)
        }
    }

    private contains(key: string): boolean {
        return this.assets.has(key)
    }

    private get(key: string): SharedArrayBuffer | undefined {
        return this.assets.get(key)
    }

    private async put(key: string, url: string): Promise<SharedArrayBuffer> {
        if (!this.assets.has(key)) {
            const shared_buffer = await fetch_into_shared_buffer(url)
            this.assets.set(key, shared_buffer)
        }

        // SAFETY: at this point it's either contained or we've just inserted it
        return this.assets.get(key)!
    }
}