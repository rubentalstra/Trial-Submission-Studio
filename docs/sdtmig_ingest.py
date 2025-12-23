import re

from llama_index.core import (
    Document,
    Settings,
    VectorStoreIndex,
)
from llama_index.core.node_parser import SentenceSplitter
from llama_index.embeddings.huggingface import HuggingFaceEmbedding
from llama_index.readers.file import PyMuPDFReader

PDF_PATH = "docs/SDTMIG v3.4-FINAL_2022-07-21.pdf"
INDEX_DIR = "docs/sdtmig_index"

# Matches: 6.2.1 Adverse Events (AE)
SECTION_RE = re.compile(r"^(\d+(?:\.\d+)+)\s+(.+)$", re.MULTILINE)


def extract_sections(text: str) -> list[tuple[str, str]]:
    """
    Split page text into (section_title, section_text).
    Falls back to 'UNSPECIFIED' if no section headers exist.
    """
    matches = list(SECTION_RE.finditer(text))

    if not matches:
        return [("UNSPECIFIED", text.strip())]

    sections: list[tuple[str, str]] = []

    for i, match in enumerate(matches):
        start = match.end()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)

        section_id = match.group(1)
        section_name = match.group(2).strip()
        title = f"{section_id} {section_name}"

        body = text[start:end].strip()
        if body:
            sections.append((title, body))

    return sections


def main() -> None:
    # 🔒 Deterministic local embeddings
    Settings.embed_model = HuggingFaceEmbedding(
        model_name="sentence-transformers/all-MiniLM-L6-v2"
    )

    reader = PyMuPDFReader()
    pages = reader.load(file_path=PDF_PATH)

    if not pages:
        raise RuntimeError("No pages loaded from SDTMIG PDF")

    documents: list[Document] = []

    # ✅ ENUMERATE for reliable page numbers
    for page_index, page in enumerate(pages, start=1):
        page_number = page_index
        text = page.text or ""

        if len(text.strip()) < 100:
            continue

        for section_title, section_text in extract_sections(text):
            # Filter out ultra-short / bullet-only noise
            if len(section_text) < 300:
                continue

            documents.append(
                Document(
                    text=section_text,
                    metadata={
                        "source": "SDTMIG v3.4",
                        "page": page_number,
                        "section": section_title,
                    },
                )
            )

    splitter = SentenceSplitter(
        chunk_size=800,
        chunk_overlap=200,
        paragraph_separator="\n\n",
    )

    index = VectorStoreIndex.from_documents(
        documents,
        transformations=[splitter],
    )

    index.storage_context.persist(persist_dir=INDEX_DIR)

    print(f"✅ SDTMIG indexed successfully ({len(documents)} sections)")


if __name__ == "__main__":
    main()
