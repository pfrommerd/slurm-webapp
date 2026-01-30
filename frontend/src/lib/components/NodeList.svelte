<script lang="ts">
    import type { Node } from "$lib/types";
    import Card from "./ui/Card.svelte";
    import Badge from "./ui/Badge.svelte";

    export let nodes: Node[] = [];

    function getStateVariant(state: string) {
        if (state.includes("idle")) return "success";
        if (state.includes("down") || state.includes("drain")) return "danger";
        if (state.includes("alloc") || state.includes("mix")) return "warning";
        return "neutral";
    }
</script>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each nodes as node}
        <Card title={node.name}>
            <div class="flex justify-between items-start mb-2">
                <Badge variant={getStateVariant(node.state)}>{node.state}</Badge
                >
            </div>
            <div class="text-sm text-zinc-600 dark:text-zinc-400 space-y-1">
                <div class="flex justify-between">
                    <span>CPUs:</span>
                    <span class="font-medium text-zinc-900 dark:text-zinc-200"
                        >{node.cpus}</span
                    >
                </div>
                <div class="flex justify-between">
                    <span>Memory:</span>
                    <span class="font-medium text-zinc-900 dark:text-zinc-200"
                        >{node.real_memory} MB</span
                    >
                </div>
                {#if node.features.length > 0}
                    <div
                        class="mt-2 pt-2 border-t border-zinc-100 dark:border-zinc-700"
                    >
                        <span class="text-xs text-zinc-500"
                            >Features: {node.features.join(", ")}</span
                        >
                    </div>
                {/if}
            </div>
        </Card>
    {/each}
</div>
