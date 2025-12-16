from llama_index.core import (
    StorageContext,
    load_index_from_storage,
)
from llama_index.embeddings.huggingface import HuggingFaceEmbedding
from llama_index.core import Settings

INDEX_DIR = "docs/sdtmig_index"


def query_sdtmig(question: str) -> None:
    # Embeddings only (no LLM)
    Settings.embed_model = HuggingFaceEmbedding(
        model_name="sentence-transformers/all-MiniLM-L6-v2"
    )
    Settings.llm = None  # 🚫 disable LLM completely

    storage = StorageContext.from_defaults(persist_dir=INDEX_DIR)
    index = load_index_from_storage(storage)

    retriever = index.as_retriever(similarity_top_k=5)
    nodes = retriever.retrieve(question)

    print("\n=== Retrieved SDTMIG Source Text ===")
    for n in nodes:
        print(f"\n[Page {n.metadata.get('page')}, {n.metadata.get('section')}]")
        print(n.text[:1500])  # cap output


if __name__ == "__main__":
    while True:
        q = input("\nAsk SDTMIG (or 'exit'): ")
        if q.lower() == "exit":
            break
        query_sdtmig(q)
