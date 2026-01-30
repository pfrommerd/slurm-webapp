<script lang="ts">
    import type { Partition } from "$lib/types";
    import Table from "./ui/Table.svelte";
    import Badge from "./ui/Badge.svelte";

    export let partitions: Partition[] = [];

    function getStateVariant(state: string) {
        if (state === "UP") return "success";
        if (state === "DOWN") return "danger";
        return "neutral";
    }
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-700"
>
    <Table>
        <thead class="bg-zinc-50 dark:bg-zinc-900/50">
            <tr>
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Partition</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >State</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Total Nodes</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Total CPUs</th
                >
            </tr>
        </thead>
        <tbody
            class="bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700"
        >
            {#each partitions as part}
                <tr>
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm font-medium text-zinc-900 dark:text-zinc-100"
                        >{part.name}</td
                    >
                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                        <Badge variant={getStateVariant(part.state)}
                            >{part.state}</Badge
                        >
                    </td>
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{part.total_nodes}</td
                    >
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{part.total_cpus}</td
                    >
                </tr>
            {/each}
            {#if partitions.length === 0}
                <tr>
                    <td
                        colspan="4"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No partitions found</td
                    >
                </tr>
            {/if}
        </tbody>
    </Table>
</div>
