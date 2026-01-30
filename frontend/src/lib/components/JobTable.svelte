<script lang="ts">
    import type { Job } from "$lib/types";
    import { JobState } from "$lib/types";
    import Table from "./ui/Table.svelte";
    import TableHeader from "./ui/TableHeader.svelte";
    import TableBody from "./ui/TableBody.svelte";
    import TableRow from "./ui/TableRow.svelte";
    import TableHead from "./ui/TableHead.svelte";
    import TableCell from "./ui/TableCell.svelte";
    import Badge from "./ui/Badge.svelte";

    export let jobs: Job[] = [];

    function getStateVariant(state: JobState) {
        switch (state) {
            case JobState.RUNNING:
                return "success";
            case JobState.PENDING:
                return "warning";
            case JobState.FAILED:
                return "danger";
            case JobState.CANCELLED:
                return "neutral";
            case JobState.COMPLETED:
                return "success";
            default:
                return "neutral";
        }
    }

    function formatDate(dateStr: string | null) {
        if (!dateStr) return "-";
        return new Date(dateStr).toLocaleString();
    }
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-700"
>
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Job ID</TableHead>
                <TableHead>User</TableHead>
                <TableHead>Partition</TableHead>
                <TableHead>State</TableHead>
                <TableHead>Nodes</TableHead>
                <TableHead>Submit Time</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {#each jobs as job}
                <TableRow>
                    <TableCell
                        ><span
                            class="font-medium text-zinc-900 dark:text-zinc-100"
                            >{job.job_id}</span
                        ></TableCell
                    >
                    <TableCell>{job.user}</TableCell>
                    <TableCell>{job.partition}</TableCell>
                    <TableCell>
                        <Badge variant={getStateVariant(job.state)}
                            >{job.state}</Badge
                        >
                    </TableCell>
                    <TableCell>{job.num_nodes} ({job.num_cpus} CPUs)</TableCell>
                    <TableCell>{formatDate(job.submit_time)}</TableCell>
                </TableRow>
            {/each}
            {#if jobs.length === 0}
                <TableRow>
                    <td
                        colspan="6"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No active jobs</td
                    >
                </TableRow>
            {/if}
        </TableBody>
    </Table>
</div>
