import { defineConfig } from "vite";
export default defineConfig({
    // base: "/sm213_editor",
    base: "./",
    esbuild: {
        supported: {
            "top-level-await": true, //browsers can handle top-level-await features
        },
    },
});
