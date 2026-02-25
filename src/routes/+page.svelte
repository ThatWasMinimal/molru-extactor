<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {isPermissionGranted, requestPermission, sendNotification} from "@tauri-apps/plugin-notification";
  import { open, confirm } from '@tauri-apps/plugin-dialog';

  
  onMount(async () => {
    let permissionGranted = await isPermissionGranted();
    
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }
  });



  let selectedPath: string | null = null;

  async function selectFile() {
    const file = await open({
      multiple: false,
    });

    if (typeof file === "string") {
      selectedPath = file;

      try {
        const wantsCustom = await confirm(
          "Do you want to choose a custom output directory?"
        );

        let savepath: string | null = null;

        if (await isPermissionGranted()) {
          await sendNotification({
            title: "extraction in progress",
            body: "extracting your files, this may take a moment",
          });
        }

        if (wantsCustom) {
          const dir = await open({
            directory: true,
            multiple: false,
          });

          if (typeof dir === "string") {
            savepath = dir;
          }
        }

        const result = await invoke("extract_batch", {
          paths: [selectedPath],
          outputRoot: savepath, 
        });

        if (await isPermissionGranted()) {
          await sendNotification({
            title: "done",
            body: `extraction complete! your files have been saved to ${savepath ?? "lr_extracted"}`,
          });
        }

        console.log("Extraction result:", result);
      } catch (error) {
        console.error("Extraction failed:", error);
      }
    }
  }

</script>



<div class="flex flex-col min-h-screen">
  <div class="flex flex-col items-center justify-center flex-1">
    <h1 class="text-white mb-4 text-2xl font-mono">lr.</h1>
    <h1 class="text-white mb-4 font-mono">
      drop in a .molru or a .bundle file to get started
    </h1>

    <button
      on:click={selectFile}
      class="bg-stone-950 text-white px-3 py-2 border border-white rounded-lg text-sm mb-3"
    >
      Choose File
    </button>


  </div>

  <footer class="flex items-center justify-center py-4">
    <h1 class="text-white text-sm font-mono">
      made using tauri + sveltekit · by mal
    </h1>
  </footer>
</div>