export class AssetStore {
    private scene_base_url: string
    private mesh_base_url: string

    private mesh_cache: Map<string, Uint8Array>
    private scene_cache: Map<string, Uint8Array>

    constructor(scene_base_url: string, mesh_base_url: string) {
        this.scene_base_url = scene_base_url;
        this.mesh_base_url = mesh_base_url;
        this.mesh_cache = new Map()
        this.scene_cache = new Map()
    }

    async prefetch_scene_meshes(scene_file_name: string) {
        const scene_buffer = await this.load_scene(scene_file_name)
        const scene_string = new TextDecoder().decode(scene_buffer)
        const scene_js = JSON.parse(scene_string)
        const meshes = scene_js.meshes
        if (meshes != undefined) {
            for (const mesh of meshes) {
                await this.load_mesh(mesh.file_name)
            }
        }
    }

    async load_mesh(mesh_file_name: string): Promise<Uint8Array> {
        if (this.mesh_cache.has(mesh_file_name)) {
            return this.get_mesh(mesh_file_name)
        } else {
            const mesh_url = this.mesh_base_url + '/' + mesh_file_name
            const mesh = await this.fetch_into_array(mesh_url)
            console.log(`Loaded mesh '${mesh_file_name}'  size ${mesh.length}`)
            this.mesh_cache.set(mesh_file_name, mesh)
            return mesh
        }
    }

    async load_scene(scene_file_name: string): Promise<Uint8Array> {
        if (this.scene_cache.has(scene_file_name)) {
            return this.get_scene(scene_file_name)
        } else {
            const url = this.scene_base_url + '/' + scene_file_name
            const scene = await this.fetch_into_array(url)
            console.log(`Loaded scene '${scene_file_name}'  size ${scene.length}`)
            this.scene_cache.set(scene_file_name, scene)
            return scene
        }
    }

    get_mesh(mesh_file_name: string): Uint8Array {
        const mesh = this.mesh_cache.get(mesh_file_name)
        if (mesh == undefined) {
            throw new Error(`AssetStore: Mesh ${mesh_file_name} is undefined`)
        }
        console.log(`get_mesh(${mesh_file_name}): size=${mesh.length}`)
        return mesh
    }

    get_scene(scene_file_name: string): Uint8Array {
        const scene = this.scene_cache.get(scene_file_name)
        if (scene == undefined) {
            throw new Error(`AssetStore: Scene ${scene_file_name} is undefined`)
        }
        console.log(`get_scene(${scene_file_name}): size=${scene.length}`)
        return scene

    }

    private async fetch_into_array(path: string): Promise<Uint8Array> {
        const buffer = await (await fetch(path)).arrayBuffer()
        return new Uint8Array(buffer)
    }
}