<script lang="ts">
    import type { Partition } from "$lib/types";
    import Table from "./ui/Table.svelte";
    import TableHeader from "./ui/TableHeader.svelte";
    import TableBody from "./ui/TableBody.svelte";
    import TableRow from "./ui/TableRow.svelte";
    import TableHead from "./ui/TableHead.svelte";
    import TableCell from "./ui/TableCell.svelte";
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
        <TableHeader>
            <TableRow>
                <TableHead>Partition</TableHead>
                <TableHead>State</TableHead>
                <TableHead>Total Nodes</TableHead>
                <TableHead>Total CPUs</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {#each partitions as part}
                <TableRow>
                    <TableCell
                        ><span
                            class="font-medium text-zinc-900 dark:text-zinc-100"
                            >{part.name}</span
                        ></TableCell
                    >
                    <TableCell>
                        <Badge variant={getStateVariant(part.state)}
                            >{part.state}</Badge
                        >
                    </TableCell>
                    <TableCell>{part.total_nodes}</TableCell>
                    <TableCell>{part.total_cpus}</TableCell>
                </TableRow>
            {/each}
            {#if partitions.length === 0}
                <TableRow>
                    <td
                        colspan="4"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No partitions found</td
                    >
                </TableRow>
            {/if}
        </TableBody>
    </Table>
</div>
