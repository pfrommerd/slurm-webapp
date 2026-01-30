export async function fetchStatus() {
    const response = await fetch('http://localhost:3000/api/status')
    return response.json()
}

export async function fetchNodes() {
    const res = await fetch('http://localhost:3000/api/nodes')
    return res.json()
}

export async function fetchJobs() {
    const res = await fetch('http://localhost:3000/api/jobs')
    return res.json()
}
