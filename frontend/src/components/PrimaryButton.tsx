type Props = {
  text: string;
  onClick?: () => void,
  href?: string;
  target?: string
};

export const PrimaryButton = (props: Props) => {
  if(props.href != null) {
    return (
      <a
        href={props.href}
        target={props.target ?? "_self"}
        className="ml-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-lime-600 hover:bg-lime-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-lime-500"
      >
        {props.text}
      </a>
    )
  }

  return (
    <button
      type="submit"
      className="ml-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-lime-600 hover:bg-lime-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-lime-500"
      onClick={props.onClick}
    >
      {props.text}
    </button>
  );
};
