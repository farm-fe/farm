import Highlighter from 'react-highlight-words';

// import Highlighter from 'react-highlight-words/dist/main.js';
interface Props {
    text?: string;
}
function HighlightKeyword({ text }: Props) {
    return (
        <Highlighter
            highlightClassName="YourHighlightClass"
            searchWords={["and", "or", "the"]}
            autoEscape={true}
            textToHighlight="The dog is chasing the cat. Or perhaps they're just playing?"
        />
    );
}

export default HighlightKeyword;
