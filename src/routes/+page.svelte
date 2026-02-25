<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
  } from "@tauri-apps/plugin-notification";
  import {  openUrl } from '@tauri-apps/plugin-opener';
  import { open, confirm } from "@tauri-apps/plugin-dialog";

  let buildInfo = "";

  onMount(async () => {
    let permissionGranted = await isPermissionGranted();

    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === "granted";
    }

    buildInfo = await invoke<string>("build_info");
  });


  let selectedPath: string | null = null;

  let files: string[] = [];

  async function selectFile() {
    const selected = await open({
      multiple: true,
    });

    if (!selected) return;

    if (Array.isArray(selected)) {
      files = [...new Set([...files, ...selected])];
    } else {
      files = [...new Set([...files, selected])];
    }
  }

  async function extract(files: string[] | null) {
    if (files && files.length > 0) {
      selectedPath = files[0];

      try {
        const wantsCustom = await confirm(
          "Do you want to choose a custom output directory?",
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

    <ul class="flex flex-col items-center">
      {#each files as item}
        <li class="text-white font-mono text-sm justify-center">
          {item}
          <button
            on:click={() => {
              files = files.filter((f) => f !== item);
            }}
            class="text-red-500 text-xs mt-1"
          >
            x
          </button>
        </li>
      {:else}
        <li class="text-white font-mono text-sm">No items found.</li>
      {/each}
    </ul>

    <button
      on:click={selectFile}
      class="text-white px-2 py-1 rounded-lg text-xs mb-3"
    >
      browse
    </button>

    <button
      on:click={() => extract(files)}
      class="bg-black text-white px-3 py-2 border border-white rounded-lg text-sm font-mono hover:bg-white hover:text-black transition-colors"
    >
      extract
    </button>
  </div>

  <footer class="flex flex-col items-center justify-center py-4">
    <h1 class="text-white text-sm font-mono flex items-center ml-2">
      made using tauri + sveltekit · by mal · v{buildInfo}
    </h1>
    <button type="button" on:click={async () => await openUrl("https://github.com/ThatWasMinimal/molru-extractor")} class="text-white text-sm font-mono flex items-center ml-2 hover:italic hover:underline transition-all">
      view on github.
    </button>
  </footer>
</div>
