import { Link } from "react-router-dom"

const AboutPage = () => {
  return (
    <main className="h-screen w-screen bg-gray-100 flex flex-col gap-2 justify-center items-center p-4">
      <h1 className="text-2xl text-center">About Page</h1>

      <Link to="/" className="text-blue-500 underline">
        Home Page
      </Link>
    </main>
  )
}

export default AboutPage
