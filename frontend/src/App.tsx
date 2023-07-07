import { createBrowserRouter, RouterProvider } from "react-router-dom"

// File Based Routing: https://dev.to/franciscomendes10866/file-based-routing-using-vite-and-react-router-3fdo
const pages: any = import.meta.glob("./pages/**/*.tsx", { eager: true })

const routes = []

for (const path of Object.keys(pages)) {
  const fileName = path.match(/\.\/pages\/(.*)\.tsx$/)?.[1]
  if (!fileName) {
    continue
  }

  const normalizedPathName = fileName.includes("$")
    ? fileName.replace("$", ":")
    : fileName.replace(/\/index/, "")

  routes.push({
    path: fileName === "index" ? "/" : `/${normalizedPathName.toLowerCase()}`,
    Element: pages[path].default,
    ErrorBoundary: pages[path]?.ErrorBoundary,
  })
}

const router = createBrowserRouter(
  routes.map(({ Element, ErrorBoundary, ...rest }) => ({
    ...rest,
    element: <Element />,
    ...(ErrorBoundary && { errorElement: <ErrorBoundary /> }),
  }))
)

const App = () => {
  return <RouterProvider router={router} />
}

export default App
