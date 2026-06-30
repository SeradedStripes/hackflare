import NotFound from "./404"

export function loader() {
  throw new Response(null, { status: 404, statusText: "Not Found" })
}

export function ErrorBoundary() {
  return <NotFound />
}
