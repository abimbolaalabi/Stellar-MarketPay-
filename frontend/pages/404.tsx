import Link from "next/link";

import { useTranslation } from "@/lib/i18n";

export default function Custom404() {
    const { t } = useTranslation("common");

    return (
        <main className="min-h-screen flex items-center justify-center text-white px-6">
            <div className="text-center w-full max-w-2xl ">
                <h1 className="text-8xl md:text-9xl font-display font-black bg-gradient-to-r from-market-400 via-market-600 to-market-400 bg-clip-text text-transparent lg:text-[12rem]">
                    404
                </h1>

                <h2 className="mt-6 text-2xl md:text-3xl font-semibold">
                    {t("notFound.title")}
                </h2>

                <p className="mt-4 text-amber-100 text-base md:text-lg">
                    {t("notFound.description")}
                </p>

                <div className="mt-8 flex flex-col sm:flex-row gap-4 justify-center">
                    <Link href="/" className="btn-primary text-base px-8 py-3.5 inline-flex justify-center">
                        {t("notFound.goHome")}
                    </Link>

                    <Link href="/jobs" className="btn-secondary text-base px-8 py-3.5 inline-flex justify-center">
                        {t("notFound.browseJobs")}
                    </Link>
                </div>
            </div>
        </main>
    );
}
