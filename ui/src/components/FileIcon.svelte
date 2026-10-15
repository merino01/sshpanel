<!-- El icono de un archivo o carpeta (ver lib/fileIcon): la carpeta, rellena y en el color de acento. -->
<script lang="ts">
  import type { Component } from "svelte";
  import Archive from "~icons/heroicons/archive-box";
  import Code from "~icons/heroicons/code-bracket";
  import File from "~icons/heroicons/document";
  import Text from "~icons/heroicons/document-text";
  import Folder from "~icons/heroicons/folder-solid";
  import Link from "~icons/heroicons/link";
  import Image from "~icons/heroicons/photo";
  import type { Entry } from "../lib/api";
  import { fileIcon, type FileIconKind } from "../lib/fileIcon";

  let { entry }: { entry: Pick<Entry, "name" | "kind"> } = $props();

  const ICONS: Record<FileIconKind, Component> = { folder: Folder, link: Link, image: Image, code: Code, archive: Archive, text: Text, file: File };
  const kind = $derived(fileIcon(entry));
  const Icon = $derived(ICONS[kind]);
</script>

<Icon class="size-[18px] shrink-0 group-data-selected:text-primary-foreground {kind === 'folder' ? 'text-primary' : 'text-muted-foreground'}" aria-hidden="true" />
