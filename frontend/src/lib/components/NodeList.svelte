<script lang="ts">
    import { type Node, NodeStatus } from "$lib/types";
    import TableUI from "./ui/Table.svelte";
    import TableHeader from "./ui/TableHeader.svelte";
    import TableBody from "./ui/TableBody.svelte";
    import TableRow from "./ui/TableRow.svelte";
    import TableHead from "./ui/TableHead.svelte";
    import TableCell from "./ui/TableCell.svelte";
    import Badge from "./ui/Badge.svelte";

    export let nodes: Node[] = [];

    $: sortedNodes = [...nodes].sort((a, b) => a.name.localeCompare(b.name));

    function getStateVariant(state: NodeStatus) {
        if (state === NodeStatus.Idle) return "success";
        if (state === NodeStatus.Down) return "danger";
        if (state === NodeStatus.Mix) return "warning";
        return "neutral";
    }
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-700"
>
    <TableUI>
        <TableHeader>
            <TableRow>
                <TableHead>Node Name</TableHead>
                <TableHead>State</TableHead>
                <TableHead>CPUs</TableHead>
                <TableHead>Memory</TableHead>
                <TableHead>Resources</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {#each sortedNodes as node}
                <TableRow>
                    <TableCell
                        ><span
                            class="font-medium text-zinc-900 dark:text-zinc-100"
                            >{node.name}</span
                        ></TableCell
                    >
                    <TableCell>
                        <Badge variant={getStateVariant(node.status)}
                            >{node.status}</Badge
                        >
                    </TableCell>
                    <TableCell>{node.cpus}</TableCell>
                    <TableCell>{node.memory} MB</TableCell>
                    <TableCell>
                        {#if node.resources.size > 0}
                            <div class="flex flex-wrap gap-1">
                                {#each [...node.resources.values()] as res}
                                    <Badge variant="neutral" size="sm">
                                        {res.resource}: {res.available}/{res.total}
                                    </Badge>
                                {/each}
                            </div>
                        {:else}
                            <span class="text-zinc-400">-</span>
                        {/if}
                    </TableCell>
                </TableRow>
            {/each}
            {#if sortedNodes.length === 0}
                <TableRow>
                    <td
                        colspan="5"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No nodes found</td
                    >
                </TableRow>
            {/if}
        </TableBody>
    </TableUI>
</div>
