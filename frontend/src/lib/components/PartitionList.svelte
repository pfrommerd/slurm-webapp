<script lang="ts">
    import { PartitionStatus, type Partition } from "$lib/types";
    import TableUI from "./ui/Table.svelte";
    import TableHeader from "./ui/TableHeader.svelte";
    import TableBody from "./ui/TableBody.svelte";
    import TableRow from "./ui/TableRow.svelte";
    import TableHead from "./ui/TableHead.svelte";
    import TableCell from "./ui/TableCell.svelte";
    import Badge from "./ui/Badge.svelte";

    export let partitions: Partition[] = [];

    $: sortedPartitions = [...partitions].sort((a, b) =>
        a.name.localeCompare(b.name),
    );

    function getStateVariant(state: PartitionStatus) {
        if (state === PartitionStatus.Up) return "success";
        if (state === PartitionStatus.Down) return "danger";
        return "neutral";
    }
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-700"
>
    <TableUI>
        <TableHeader>
            <TableRow>
                <TableHead>Partition</TableHead>
                <TableHead>State</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {#each sortedPartitions as part}
                <TableRow>
                    <TableCell
                        ><span
                            class="font-medium text-zinc-900 dark:text-zinc-100"
                            >{part.name}</span
                        ></TableCell
                    >
                    <TableCell>
                        <Badge variant={getStateVariant(part.status)}
                            >{part.status}</Badge
                        >
                    </TableCell>
                </TableRow>
            {/each}
            {#if sortedPartitions.length === 0}
                <TableRow>
                    <td
                        colspan="2"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No partitions found</td
                    >
                </TableRow>
            {/if}
        </TableBody>
    </TableUI>
</div>
