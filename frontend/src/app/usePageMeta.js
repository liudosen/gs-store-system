import { routeMeta } from "./routes";

function ensureMeta(selector, attributes) {
  let element = document.head.querySelector(selector);
  if (!element) {
    element = document.createElement("meta");
    document.head.appendChild(element);
  }

  Object.entries(attributes).forEach(([key, value]) => {
    element.setAttribute(key, value);
  });
}

export function usePageMeta() {
  function updatePageMeta(path) {
    const meta = routeMeta[path] || routeMeta["/"];
    const imageUrl = new URL(meta.image, window.location.origin).href;

    document.title = meta.title;
    ensureMeta('meta[name="description"]', {
      name: "description",
      content: meta.description,
    });
    ensureMeta('meta[property="og:type"]', {
      property: "og:type",
      content: "website",
    });
    ensureMeta('meta[property="og:title"]', {
      property: "og:title",
      content: meta.title,
    });
    ensureMeta('meta[property="og:description"]', {
      property: "og:description",
      content: meta.description,
    });
    ensureMeta('meta[property="og:image"]', {
      property: "og:image",
      content: imageUrl,
    });
    ensureMeta('meta[property="og:url"]', {
      property: "og:url",
      content: window.location.href,
    });
    ensureMeta('meta[name="twitter:card"]', {
      name: "twitter:card",
      content: "summary_large_image",
    });
    ensureMeta('meta[name="twitter:title"]', {
      name: "twitter:title",
      content: meta.title,
    });
    ensureMeta('meta[name="twitter:description"]', {
      name: "twitter:description",
      content: meta.description,
    });
    ensureMeta('meta[name="twitter:image"]', {
      name: "twitter:image",
      content: imageUrl,
    });
  }

  return {
    updatePageMeta,
  };
}
