import { readFile } from "fs/promises";
import { bucketActor, canisterId } from "./bucket.actor.mjs";

const uploadHtml = async ({ name, folder, src, fullPath, token, mimeType }) => {
  const buffer = await readFile(src);

  const { batchId } = await bucketActor.initUpload({
    name,
    mimeType,
    fullPath,
    token: token ? [token] : [],
    folder,
    sha256: [],
  });

  console.log(`[${name}] Init.`);

  const chunkSize = 700000;
  const promises = [];

  const upload = async (chunks) => {
    console.log(`Uploading asset in: ${canisterId}`);

    const { chunkId } = await bucketActor.uploadChunk({
      batchId,
      content: [...new Uint8Array(chunks)],
    });

    console.log(`[${name} - ${chunkId}] Chunk.`);

    return chunkId;
  };

  for (let start = 0; start < buffer.length; start += chunkSize) {
    const chunks = buffer.slice(start, start + chunkSize);
    promises.push(upload(chunks));
  }

  const chunkIds = await Promise.all(promises);

  console.log(`[${name}] Chunks.`);

  await bucketActor.commitUpload({
    batchId,
    chunkIds: chunkIds,
    headers: [
      ["Content-Type", mimeType],
      ["accept-ranges", "bytes"],
      ...[["Cache-Control", `max-age=0`]],
    ],
  });

  console.log(`[${name}] Commit.`);
};

const uploadAssets = async () => {
  await Promise.all([
    uploadHtml({
      src: "./data/index.html",
      name: "index.html",
      folder: "resources",
      fullPath: "/",
      mimeType: "text/html"
    }),
    uploadHtml({
      src: "./data/post.html",
      name: "post1234",
      folder: "d",
      fullPath: "/d/post1234",
      mimeType: "text/html"
    }),
    uploadHtml({
      src: "./data/sample_1920x1280.png",
      name: "sample_1920x1280.png",
      folder: "images",
      fullPath: "/images/sample_1920x1280.png",
      token: "123",
      mimeType: "image/jpeg"
    }),
  ]);
};

uploadAssets().then(() => {
  console.log("Done.");
});
