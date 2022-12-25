#pragma once
#include "data/resource.hh"
#include "net/net.hh"

namespace mcvm {
	// A mcvm package
	class Package {
		protected:

		const std::string name;
		fs::path location;
		// Contents of the package build script
		std::string contents;

		Package(const std::string& _name, const fs::path& _location);

		public:
		// Ensure that the package contents are stored in memory
		virtual void ensure_contents() {}

		virtual ~Package() = default;
	};

	// A package installed from the internet, which has more restrictions
	class RemotePackage : public Package {
		const std::string url;

		public:
		RemotePackage(const std::string& _name, const std::string& _url, const fs::path& cache_dir);

		void ensure_contents() override;
	};

	// A package installed from the local filesystem
	class LocalPackage : public Package {
		public:
		LocalPackage(const std::string& _name, const fs::path& path);

		void ensure_contents() override;
	};
};