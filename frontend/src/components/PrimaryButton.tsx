type Props = {
  text: string;
};

export const PrimaryButton = (props: Props) => {
  return (
    <button
      type="submit"
      className="ml-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-lime-600 hover:bg-lime-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-lime-500"
    >
      {props.text}
    </button>
  );
};
