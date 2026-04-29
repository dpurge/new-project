import { Body, ErrorBoundary, FileRoutes, Head, Html, Meta, Routes, Scripts, Title } from "@solidjs/start";
import { Suspense } from "solid-js";

export default function App() {
  return (
    <Html lang="en">
      <Head>
        <Title>{{ cookiecutter.site_title }}</Title>
        <Meta charset="utf-8" />
        <Meta
          name="viewport"
          content="width=device-width, initial-scale=1"
        />
        <Meta name="description" content="{{ cookiecutter.description }}" />
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
      </Head>
      <Body>
        <ErrorBoundary>
          <Suspense>
            <Routes>
              <FileRoutes />
            </Routes>
          </Suspense>
        </ErrorBoundary>
        <Scripts />
      </Body>
    </Html>
  );
}
