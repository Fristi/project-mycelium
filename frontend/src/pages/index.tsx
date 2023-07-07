import { Link } from "react-router-dom"

const HomePage = () => {
  return (
    <main className="h-screen w-screen bg-gray-100 flex flex-col gap-2 justify-center items-center p-4">
      <h1 className="text-2xl text-center">
        Capacitorjs + React + Tailwind Template
      </h1>
      <h3 className="text-center">
        A template to get started with capacitor js, react and tailwind to
        create mobile app
      </h3>
      <Link to="/about" className="text-blue-500 underline">
        About Page
      </Link>
    </main>
  )
}

export default HomePage
