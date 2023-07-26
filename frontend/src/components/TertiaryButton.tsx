type Props = {
    text: string;
    href: string
}

export const TertiaryButton = (props: Props) => {
    return (
        <a
        href={props.href}
        className="bg-white py-2 px-4 text-sm font-medium text-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
      >
        Cancel
      </a>
    )
}