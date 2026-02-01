<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { fetchState } from "$lib/api";
    import type { ClusterState } from "$lib/types"; // Import from types now
    import NodeList from "$lib/components/NodeList.svelte";
    import JobTable from "$lib/components/JobTable.svelte";
    import PartitionList from "$lib/components/PartitionList.svelte";
    import Badge from "$lib/components/ui/Badge.svelte";

    let state: ClusterState | null = null;
    let error: string | null = null;
    let interval: ReturnType<typeof setInterval>;

    async function refresh() {
        try {
            state = await fetchState();
            console.log(state);
            error = null;
        } catch (e) {
            console.error(e);
            error = "Failed to connect to backend";
        }
    }

    onMount(() => {
        refresh();
        interval = setInterval(refresh, 5000); // Auto-refresh every 5s
    });

    onDestroy(() => {
        if (interval) clearInterval(interval);
    });
</script>

<div class="space-y-8">
    <div class="flex justify-between items-center">
        <div>
            <h1
                class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight"
            >
                SLURM Dashboard
            </h1>
            {#if state && state.updated_at}
                <p class="text-sm text-zinc-500 mt-1">
                    Last updated: {new Date(
                        state.updated_at,
                    ).toLocaleTimeString()}
                </p>
            {/if}
        </div>
        {#if error}
            <Badge variant="danger">{error}</Badge>
        {:else if state}
            <Badge variant="success">Connected</Badge>
        {:else}
            <Badge variant="warning">Connecting...</Badge>
        {/if}
    </div>

    {#if state}
        <section>
            <h2
                class="text-xl font-semibold text-zinc-900 dark:text-zinc-100 mb-4"
            >
                Cluster Partitions
            </h2>
            <PartitionList partitions={state.partitions} />
        </section>

        <section>
            <h2
                class="text-xl font-semibold text-zinc-900 dark:text-zinc-100 mb-4"
            >
                Compute Nodes
            </h2>
            <NodeList nodes={state.nodes} />
        </section>

        <section>
            <h2
                class="text-xl font-semibold text-zinc-900 dark:text-zinc-100 mb-4"
            >
                Active Jobs
            </h2>
            <JobTable jobs={state.jobs} />
        </section>
    {:else if !error}
        <div class="text-center py-12">
            <div
                class="animate-spin rounded-full h-12 w-12 border-b-2 border-zinc-900 dark:border-white mx-auto"
            ></div>
            <p class="mt-4 text-zinc-600 dark:text-zinc-400">
                Loading cluster status...
            </p>
        </div>
    {/if}
</div>
