cmake_minimum_required (VERSION 3.3)

project( NFIQ2_SUPERBUILD )

string(REPLACE ";" "$<SEMICOLON>" EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES "${CMAKE_OSX_ARCHITECTURES}")

message(STATUS "NFIQ 2 Superbuild")

option(BUILD_NFIQ2_CLI "Build the Command-line Interface for NFIQ2" ON)

# Options for embedding random forest parameters
option(EMBED_RANDOM_FOREST_PARAMETERS "Embed random forest parameters in library" OFF)
set(EMBEDDED_RANDOM_FOREST_PARAMETER_FCT "0" CACHE STRING
    "ANSI/NIST-ITL 1-2011: Update 2015 friction ridge capture technology (FRCT) code for parameters to embed")
set(EMBEDDING_CMAKE_ARGS -DEMBEDDED_RANDOM_FOREST_PARAMETER_FCT=${EMBEDDED_RANDOM_FOREST_PARAMETER_FCT})
if(EMBED_RANDOM_FOREST_PARAMETERS)
	message(STATUS "Embedding random forest parameters")
	list(APPEND EMBEDDING_CMAKE_ARGS -DEMBED_RANDOM_FOREST_PARAMETERS=${EMBED_RANDOM_FOREST_PARAMETERS})
endif()

# macOS Code Signing
option(MACOS_CODESIGN "Sign the macOS binaries and package installers" OFF)
set(MACOS_APPLICATION_SIGNING_IDENTITY "" CACHE STRING "Apple Developer ID for applications")
set(MACOS_INSTALLER_SIGNING_IDENTITY "" CACHE STRING "Apple Developer ID for installers")
set(MACOS_NOTARYTOOL_PROFILE "" CACHE STRING "Name of notarytool credential profile stored in Keychain (insert these with `xcrun notarytool store-credentials')")

option(MACOS_FORCE_STATIC "Force static library dependencies only" OFF)

set( NO_SEARCH TRUE )

set( ROOT_PATH ${PROJECT_SOURCE_DIR} )
set( BUILD_PATH  ${CMAKE_CURRENT_BINARY_DIR} )

# ExternalProjects have their own install() that we can't control, and the
# `install' target is executed at superbuild build time. To counteract this, we
# will set CMAKE_INSTALL_PREFIX to ${INSTALL_STAGING_DIR} for all
# ExternalProjects. The superbuild's install and  packaging with CPack will copy
# this staging directory.
set(NFIQ2_CONTAINER_DIR "nfiq2")
set(INSTALL_STAGING_DIR "${CMAKE_CURRENT_BINARY_DIR}/install_staging/${NFIQ2_CONTAINER_DIR}")

set(CMAKE_CXX_STANDARD 11)

# set colors for cmake comand line
include( "${ROOT_PATH}/cmake/colors.cmake" )

# detect target platform
include( "${ROOT_PATH}/cmake/target.cmake" )

# setup compiler
include( "${ROOT_PATH}/cmake/compiler.cmake" )

# include special settings for fingerjetfxose
include( "${ROOT_PATH}/cmake/fingerjetfxose.cmake" )

add_subdirectory("${ROOT_PATH}/fingerjetfxose/FingerJetFXOSE/libFRFXLL/src/" "${BUILD_PATH}/fingerjetfxose/FingerJetFXOSE/libFRFXLL/src/")

# forwarding 32/64 compiler flags
foreach(flag_var
        CMAKE_C_FLAGS CMAKE_C_FLAGS_DEBUG CMAKE_C_FLAGS_RELEASE CMAKE_C_FLAGS_MINSIZEREL CMAKE_C_FLAGS_RELWITHDEBINFO
        CMAKE_CXX_FLAGS CMAKE_CXX_FLAGS_DEBUG CMAKE_CXX_FLAGS_RELEASE CMAKE_CXX_FLAGS_MINSIZEREL CMAKE_CXX_FLAGS_RELWITHDEBINFO)
			if (32BITS)
				set(${flag_var} "${${flag_var}} -m32")
			elseif(64BITS)
				set(${flag_var} "${${flag_var}} -m64")
			endif()
endforeach(flag_var)

# forwarding android build flags
if( NOT "${ANDROID_ABI}" STREQUAL "")
  message( STATUS "${Gn}-- Forwarding Android build flags to the external projects${Na}" )
  set(ANDROID_CMAKE_ARGS
    -DANDROID_ABI=${ANDROID_ABI}
    -DANDROID_PLATFORM=${ANDROID_PLATFORM}
    -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}
    -DCMAKE_ANDROID_NDK=${CMAKE_ANDROID_NDK}
    -DANDROID_TARGET=${ANDROID_TARGET}
  )
endif()

# forwarding iOS build flags
if( APPLE AND NOT ( "${CMAKE_TOOLCHAIN_FILE}" STREQUAL ""))
  message( STATUS "${Gn}-- Forwarding IOS build flags to the external projects${Na}" )
  set(IOS_CMAKE_ARGS
    -DPLATFORM=${PLATFORM}
    -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}
    -DENABLE_ARC=${ENABLE_ARC}
    -DENABLE_BITCODE=${ENABLE_BITCODE}
    -DCMAKE_SYSTEM_PROCESSOR=${CMAKE_SYSTEM_PROCESSOR}
  )
endif()

include(ExternalProject)

set(default_build_type "Release")
if (NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
	message(STATUS "Setting build/configuration to ${default_build_type}")
	if (${CMAKE_VERSION} VERSION_GREATER "3.8.99999")
		get_cmake_property(IS_MULTI_CONFIG GENERATOR_IS_MULTI_CONFIG)
		if (${IS_MULTI_CONFIG})
			set(CMAKE_CONFIGURATION_TYPES "${default_build_type}" CACHE STRING "Build configurations")
			set_property(CACHE CMAKE_CONFIGURATION_TYPES PROPERTY STRINGS "Debug" "Release")
		else()
			set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE STRING "Build configuration")
			set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
		endif()
	elseif (MSVC OR Xcode)
		set(CMAKE_CONFIGURATION_TYPES "${default_build_type}" CACHE STRING "Build configurations")
		set_property(CACHE CMAKE_CONFIGURATION_TYPES PROPERTY STRINGS "Debug" "Release")
	else()
		set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE STRING "Build configuration")
		set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
	endif()
endif ()

set(MULTI_CONFIG_ARGS)
if (${CMAKE_VERSION} VERSION_GREATER "3.8.99999")
	get_cmake_property(IS_MULTI_CONFIG GENERATOR_IS_MULTI_CONFIG)
	if (${IS_MULTI_CONFIG})
		list(APPEND MULTI_CONFIG_ARGS
			-DCMAKE_CONFIGURATION_TYPES=${CMAKE_CONFIGURATION_TYPES}
			)
	else()
		list(APPEND MULTI_CONFIG_ARGS
			-DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
			)
	endif()
elseif (MSVC OR Xcode)
	list(APPEND MULTI_CONFIG_ARGS
		-DCMAKE_CONFIGURATION_TYPES=${CMAKE_CONFIGURATION_TYPES}
		)
else()
	list(APPEND MULTI_CONFIG_ARGS
		-DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
		)
endif()

set(OPENCV_CMAKE_ARGS
	-DBUILD_SHARED_LIBS=OFF
	-DBUILD_OPENCV_APPS=OFF
	-DBUILD_opencv_apps=OFF #opencv 4.5.4 seems to use this flag with lower case, keeping both entries to anticipate future fix
	-DBUILD_ANDROID_EXAMPLES=OFF
	-DBUILD_DOCS=OFF
	-DBUILD_EXAMPLES=OFF
	-DBUILD_PACKAGE=OFF
	-DBUILD_PERF_TESTS=OFF
	-DBUILD_TESTS=OFF
	-DBUILD_WITH_DEBUG_INFO=OFF
	-DBUILD_FAT_JAVA_LIB=OFF
	-DBUILD_ANDROID_PROJECTS=OFF
	-DBUILD_ANDROID_SERVICE=OFF
	-DBUILD_ANDROID_PACKAGE=OFF
	-DBUILD_TINY_GPU_MODULE=OFF
	-DENABLE_PRECOMPILED_HEADERS=OFF
	-DWITH_GTK=OFF
	-DWITH_CAROTENE=OFF
	-DENABLE_FAST_MATH=OFF
	-DWITH_PROTOBUF=OFF
	-DBUILD_PROTOBUF=OFF
	-DWITH_IPP=OFF
	-DWITH_ITT=OFF
	-DWITH_DSHOW=OFF
	-DWITH_MSMF=OFF
	-DWITH_FFMPEG=OFF
	-DWITH_EIGEN=OFF
	-DWITH_TBB=OFF
	-DWITH_OPENMP=OFF
	-DWITH_PTHREADS_PF=OFF
	-DWITH_OPENJPEG=OFF
	-DBUILD_OPENJPEG=OFF
	-DWITH_TIFF=OFF
	-DBUILD_TIFF=OFF
	-DWITH_PNG=OFF
	-DWITH_JASPER=OFF
	-DWITH_JPEG=OFF
	-DBUILD_PNG=OFF
	-DBUILD_JASPER=OFF
	-DBUILD_JPEG=OFF
	-DWITH_ADE=OFF
	-DWITH_WEBP=OFF
	-DWITH_OPENEXR=OFF
	-DWITH_QUIRC=OFF
	-DWITH_OBSENSOR=OFF
	-DBUILD_LIST=core,ml,imgproc,imgcodecs)

if("${TARGET_PLATFORM}" MATCHES "win*")
list(APPEND OPENCV_CMAKE_ARGS
	-DBUILD_WITH_STATIC_CRT=ON)
endif("${TARGET_PLATFORM}" MATCHES "win*")

if ("${TARGET_PLATFORM}" MATCHES "apple32")
	list(APPEND OPENCV_CMAKE_ARGS -DWITH_LAPACK=OFF)
endif ()

# For releases, we must not have dynamic lib dependencies from MacPorts/Homebrew
# or we'll have to redistribute them. If you're building for your own machine,
# link whatever you find. macOS 11+ hides zlib and other system libraries.
if (MACOS_FORCE_STATIC OR (APPLE AND MACOS_CODESIGN))
	list(APPEND OPENCV_CMAKE_ARGS -DBUILD_ZLIB=ON)
endif()

set(VCPKG_CMAKE_ARGS)
if (MSVC)
	list(APPEND VCPKG_CMAKE_ARGS
		-DVCPKG_VERBOSE=${VCPKG_VERBOSE}
		-DVCPKG_TARGET_TRIPLET=${VCPKG_TARGET_TRIPLET}
		)
endif()

ExternalProject_Add(OpenCV
	SOURCE_DIR ${PROJECT_SOURCE_DIR}/opencv
	CMAKE_ARGS
		${OPENCV_CMAKE_ARGS}
		${COMPILER_CMAKE_ARGS}
		${ANDROID_CMAKE_ARGS}
		${IOS_CMAKE_ARGS}
		${MULTI_CONFIG_ARGS}
		-DCMAKE_INSTALL_PREFIX=${INSTALL_STAGING_DIR}
		-DCMAKE_OSX_ARCHITECTURES=${EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES}
)

if (BUILD_NFIQ2_CLI)

# For releases, we must not have dynamic lib dependencies from MacPorts/Homebrew
# or we'll have to redistribute them. If you're building for your own machine,
# link whatever you find
if (MACOS_FORCE_STATIC OR (APPLE AND MACOS_CODESIGN))
	list(APPEND EXTRA_BIOMEVAL_CMAKE_ARGS -DFORCE_STATIC_DEPENDENCIES=YES)
endif()
if (NOT "${OPENSSL_ROOT_DIR}" STREQUAL "")
	list(APPEND EXTRA_BIOMEVAL_CMAKE_ARGS "-DOPENSSL_ROOT_DIR=${OPENSSL_ROOT_DIR}")
endif()

ExternalProject_Add(libbiomeval
	SOURCE_DIR ${PROJECT_SOURCE_DIR}/libbiomeval
	INSTALL_COMMAND ""
	CMAKE_ARGS
		-DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}
		-DBUILD_BIOMEVAL_TESTS=OFF
		-DCMAKE_INSTALL_PREFIX=${INSTALL_STAGING_DIR}
		-DBUILD_SHARED_LIBS=OFF
		-DWITH_HWLOC=OFF
		-DWITH_MPI=OFF
		-DWITH_FFMPEG=OFF
		-DWITH_PCSC=OFF
		${VCPKG_CMAKE_ARGS}
		${MULTI_CONFIG_ARGS}
		${EXTRA_BIOMEVAL_CMAKE_ARGS}
		-DCMAKE_OSX_ARCHITECTURES=${EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES}
)

ExternalProject_Add(nfir
	SOURCE_DIR ${PROJECT_SOURCE_DIR}/NFIR
	CMAKE_ARGS
		-DCMAKE_OSX_ARCHITECTURES=${EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES}
		-DCMAKE_INSTALL_PREFIX=${INSTALL_STAGING_DIR}
		-DOpenCV_DIR=${BUILD_PATH}/OpenCV-prefix/src/OpenCV-build
		-DFORCE_WIN_STATIC_RUNTIME=ON
		${MULTI_CONFIG_ARGS}
)

ExternalProject_Add_StepDependencies(nfir build OpenCV)

endif(BUILD_NFIQ2_CLI)

ExternalProject_Add(nfiq2
	SOURCE_DIR	${ROOT_PATH}/NFIQ2/NFIQ2Algorithm
	CMAKE_ARGS
		-DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}
		-DBUILD_NFIQ2_CLI=${BUILD_NFIQ2_CLI}
		-DSUPERBUILD_ROOT_PATH=${ROOT_PATH}
		-DTARGET_PLATFORM=${TARGET_PLATFORM}
		${COMPILER_CMAKE_ARGS}
		-DCMAKE_INSTALL_PREFIX=${INSTALL_STAGING_DIR}
		${ANDROID_CMAKE_ARGS}
		${IOS_CMAKE_ARGS}
		${VCPKG_CMAKE_ARGS}
		${MULTI_CONFIG_ARGS}
		${EMBEDDING_CMAKE_ARGS}
		-DCMAKE_OSX_ARCHITECTURES=${EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES}
	BUILD_ALWAYS	YES
)
ExternalProject_Add_StepDependencies(nfiq2 build OpenCV FRFXLL FRFXLL_static)

if (BUILD_NFIQ2_CLI)
	ExternalProject_Add_StepDependencies(nfiq2 build libbiomeval nfir)
endif(BUILD_NFIQ2_CLI)

ExternalProject_Add(nfiq2api
	SOURCE_DIR	${ROOT_PATH}/NFIQ2/NFIQ2Api
	CMAKE_ARGS
		-DSUPERBUILD_ROOT_PATH=${ROOT_PATH}
		${COMPILER_CMAKE_ARGS}
		${MULTI_CONFIG_ARGS}
		-DCMAKE_INSTALL_PREFIX=${INSTALL_STAGING_DIR}
		${ANDROID_CMAKE_ARGS}
		${IOS_CMAKE_ARGS}
		${EMBEDDING_CMAKE_ARGS}
		-DCMAKE_OSX_ARCHITECTURES=${EXTERNALPROJECT_SAFE_OSX_ARCHITECTURES}
	BUILD_ALWAYS	YES
)
ExternalProject_Add_StepDependencies(nfiq2api build nfiq2)

# Tell CPack to pack up the install
include(GNUInstallDirs)

# Put all files into nfiq2 container dir
set(INSTALL_STAGING_BINDIR "${CMAKE_INSTALL_BINDIR}")
set(INSTALL_STAGING_DATADIR "${CMAKE_INSTALL_DATADIR}")
set(INSTALL_STAGING_INCLUDEDIR "${CMAKE_INSTALL_INCLUDEDIR}")
set(INSTALL_STAGING_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
if (UNIX)
	set(ORIGINAL_CMAKE_INSTALL_BINDIR "${CMAKE_INSTALL_BINDIR}")
	set(ORIGINAL_CMAKE_INSTALL_DATADIR "${INSTALL_STAGING_DATADIR}")
	set(ORIGINAL_CMAKE_INSTALL_INCLUDEDIR "${INSTALL_STAGING_INCLUDEDIR}")
	set(ORIGINAL_CMAKE_INSTALL_LIBDIR "${INSTALL_STAGING_LIBDIR}")
	string(PREPEND CMAKE_INSTALL_BINDIR "${NFIQ2_CONTAINER_DIR}/")
	string(PREPEND CMAKE_INSTALL_DATADIR "${NFIQ2_CONTAINER_DIR}/")
	string(PREPEND CMAKE_INSTALL_INCLUDEDIR "${NFIQ2_CONTAINER_DIR}/")
	string(PREPEND CMAKE_INSTALL_LIBDIR "${NFIQ2_CONTAINER_DIR}/")
endif()

if (BUILD_NFIQ2_CLI)
	install(DIRECTORY "${INSTALL_STAGING_DIR}/${INSTALL_STAGING_BINDIR}/"
		DESTINATION ${CMAKE_INSTALL_BINDIR}
		FILE_PERMISSIONS
			OWNER_READ OWNER_EXECUTE
			GROUP_READ GROUP_EXECUTE
			WORLD_READ WORLD_EXECUTE
		COMPONENT nfiq2_cli)
	if (UNIX)
		install(DIRECTORY "${INSTALL_STAGING_DIR}/${INSTALL_STAGING_DATADIR}/"
			DESTINATION ${CMAKE_INSTALL_DATADIR}
			COMPONENT nfiq2_cli)
	endif()
endif(BUILD_NFIQ2_CLI)
install(DIRECTORY "${INSTALL_STAGING_DIR}/${INSTALL_STAGING_LIBDIR}/"
	DESTINATION ${CMAKE_INSTALL_LIBDIR}
	COMPONENT development)
install(DIRECTORY "${INSTALL_STAGING_DIR}/${INSTALL_STAGING_INCLUDEDIR}/"
	DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
	COMPONENT development)

include(NFIQ2/version.cmake)
set(CPACK_PACKAGE_NAME "${NFIQ2_PRODUCT_NAME}")
set(CPACK_PACKAGE_VENDOR "${NFIQ2_PRODUCT_VENDOR}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "${NFIQ2_PRODUCT_DESCRIPTION_SHORT}")
set(CPACK_COMPONENT_NFIQ2_CLI_DESCRIPTION_SUMMARY "${NFIQ2_PRODUCT_DESCRIPTION_SHORT} command-line interface")
set(CPACK_COMPONENT_DEVELOPMENT_DESCRIPTION_SUMMARY "${NFIQ2_PRODUCT_DESCRIPTION_SHORT} developer files")
set(CPACK_PACKAGE_HOMEPAGE_URL "https://github.com/usnistgov/nfiq2")
set(CPACK_PACKAGE_CONTACT "${NFIQ2_EMAIL}")
set(CPACK_PACKAGE_VERSION "${NFIQ2_VERSION}")
set(CPACK_PACKAGE_VERSION_MAJOR "${NFIQ2_VERSION_MAJOR}")
set(CPACK_PACKAGE_VERSION_MINOR "${NFIQ2_VERSION_MINOR}")
set(CPACK_PACKAGE_VERSION_PATCH "${NFIQ2_VERSION_PATCH}")

# Move repo documentation into usable form for packages
find_program(PANDOC pandoc)
if (PANDOC)
	set(DOC_FORMAT "rtf")

	# Strip remote SVGs from README
	execute_process(COMMAND grep -v ^\\[! "${ROOT_PATH}/README.md" OUTPUT_FILE "${CMAKE_CURRENT_BINARY_DIR}/README.md")
	execute_process(COMMAND sed -i.bak -e s|<img.*/>|| "${CMAKE_CURRENT_BINARY_DIR}/README.md")
	execute_process(COMMAND ${PANDOC} -f markdown_mmd -s --columns 1 -o "${CMAKE_CURRENT_BINARY_DIR}/README.${DOC_FORMAT}" "${CMAKE_CURRENT_BINARY_DIR}/README.md")
	set(CPACK_RESOURCE_FILE_README "${CMAKE_CURRENT_BINARY_DIR}/README.${DOC_FORMAT}")

	execute_process(COMMAND ${PANDOC} -f markdown_mmd -s --columns 1 -o "${CMAKE_CURRENT_BINARY_DIR}/LICENSE.${DOC_FORMAT}" "${ROOT_PATH}/LICENSE.md")
	set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_BINARY_DIR}/LICENSE.${DOC_FORMAT}")
else()
	file(COPY "${ROOT_PATH}/LICENSE.md" DESTINATION "${CMAKE_CURRENT_BINARY_DIR}")
	file(RENAME "${CMAKE_CURRENT_BINARY_DIR}/LICENSE.md" "${CMAKE_CURRENT_BINARY_DIR}/LICENSE.txt")
	set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_BINARY_DIR}/LICENSE.txt")

	file(COPY "${ROOT_PATH}/README.md" DESTINATION "${CMAKE_CURRENT_BINARY_DIR}")
	file(RENAME "${CMAKE_CURRENT_BINARY_DIR}/README.md" "${CMAKE_CURRENT_BINARY_DIR}/README.txt")
	set(CPACK_RESOURCE_FILE_README "${CMAKE_CURRENT_BINARY_DIR}/README.txt")
endif()

# Don't set "install_staging" component, because some generators will grab all
# files for install, including the externalprojects.
if (BUILD_NFIQ2_CLI)
	set(CPACK_COMPONENTS_ALL nfiq2_cli development)
else()
	set(CPACK_COMPONENTS_ALL development)
endif()
set(CPACK_COMPONENT_NFIQ2_CLI_DISPLAY_NAME "${NFIQ2_PRODUCT_NAME} Command Line Interface")
set(CPACK_COMPONENT_DEVELOPMENT_DISPLAY_NAME "${NFIQ2_PRODUCT_NAME} Development Files")

set(CPACK_GENERATOR)

if (UNIX)
	set(CPACK_PACKAGING_INSTALL_PREFIX "/usr/local")

	# Make a link (/usr/local/bin/nfiq2 -> /usr/local/nfiq2/bin/nfiq2) using
	# ln instead of cmake -E create_symlink because it does not have to be
	# an absolute path and the destination does not have to exist when
	# created.
	install(CODE "execute_process(COMMAND ${CMAKE_COMMAND} -E make_directory ${INSTALL_STAGING_DIR}/../bin)" COMPONENT nfiq2_cli)
	install(CODE "execute_process(COMMAND ln -s -f \"../nfiq2/bin/nfiq2\" \"nfiq2\" WORKING_DIRECTORY \"${INSTALL_STAGING_DIR}/../bin\")" COMPONENT nfiq2_cli)
	install(FILES "${INSTALL_STAGING_DIR}/../bin/nfiq2"
	    DESTINATION "${ORIGINAL_CMAKE_INSTALL_BINDIR}"
	    COMPONENT nfiq2_cli)
endif (UNIX)

if (APPLE)
	list(APPEND CPACK_GENERATOR "productbuild")
	execute_process(COMMAND sw_vers -productVersion OUTPUT_VARIABLE OSX_VERS)
	string(STRIP "${OSX_VERS}" OSX_VERS)
	execute_process(COMMAND uname -m OUTPUT_VARIABLE MAC_ARCH)
	string(STRIP "${MAC_ARCH}" MAC_ARCH)
	set(CPACK_PACKAGE_FILE_NAME "nfiq2-${CPACK_PACKAGE_VERSION}-macos-${OSX_VERS}-${MAC_ARCH}")
	set(CPACK_PRODUCTBUILD_IDENTIFIER "gov.nist.nfiq2")

#	set(CPACK_PRODUCTBUILD_RESOURCES_DIR "productbuild_resources")
#	file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/${CPACK_PRODUCTBUILD_RESOURCES_DIR}")
#	file(COPY "${ROOT_PATH}/cmake/nist_itl_two_color.svg" DESTINATION "${CPACK_PRODUCTBUILD_RESOURCES_DIR}")
#	# https://developer.apple.com/library/archive/documentation/DeveloperTools/Reference/DistributionDefinitionRef/Chapters/Distribution_XML_Ref.html
#	set(CPACK_PRODUCTBUILD_BACKGROUND "nist_itl_two_color.svg")
#	set(CPACK_PRODUCTBUILD_BACKGROUND_MIME_TYPE "image/svg")
#	set(CPACK_PRODUCTBUILD_BACKGROUND_ALIGNMENT "bottomleft")
#	set(CPACK_PRODUCTBUILD_BACKGROUND_SCALING "none")

	# macOS Code Signing
	if (MACOS_CODESIGN)
		if ("${MACOS_APPLICATION_SIGNING_IDENTITY}" STREQUAL "")
			message(FATAL_ERROR "You must set MACOS_APPLICATION_SIGNING_IDENTITY to your developer identity in order to code sign")
		endif()
		if ("${MACOS_INSTALLER_SIGNING_IDENTITY}" STREQUAL "")
			message(FATAL_ERROR "You must set MACOS_INSTALLER_SIGNING_IDENTITY to your developer identity in order to code sign")
		endif()

		set(CPACK_PRODUCTBUILD_IDENTITY_NAME "${MACOS_INSTALLER_SIGNING_IDENTITY}")
		set(CPACK_PKGBUILD_IDENTITY_NAME "${MACOS_INSTALLER_SIGNING_IDENTITY}")
		ExternalProject_Add_Step(nfiq2api codesign
		    DEPENDEES install
		    COMMAND codesign --prefix "${CPACK_PRODUCTBUILD_IDENTIFIER}." -f --options runtime --sign "${MACOS_APPLICATION_SIGNING_IDENTITY}" ${INSTALL_STAGING_DIR}/${INSTALL_STAGING_BINDIR}/nfiq2 ${INSTALL_STAGING_DIR}/${INSTALL_STAGING_BINDIR}/opencv_version ${INSTALL_STAGING_DIR}/${INSTALL_STAGING_LIBDIR}/libFRFXLL.dylib ${INSTALL_STAGING_DIR}/${INSTALL_STAGING_LIBDIR}/libNfiq2Api.dylib
		    COMMENT "Codesigning all binaries and dylibs...")

		# Notarize the installer
		# XXX: Use CPACK_POST_BUILD_SCRIPTS instead, since this can't depend on packaging
		if ("${MACOS_NOTARYTOOL_PROFILE}" STREQUAL "")
			message(FATAL_ERROR "You must set MACOS_NOTARYTOOL_PROFILE to the name of the notarytool credentials stored in your Keychain. Insert these with `xcrun notarytool store-credentials'.")
		endif()
		add_custom_target(notarize
		    COMMENT "Notarizing installer package..."
		    COMMAND xcrun notarytool submit "${CMAKE_CURRENT_BINARY_DIR}/${CPACK_PACKAGE_FILE_NAME}.pkg" --keychain-profile "${MACOS_NOTARYTOOL_PROFILE}" --wait
		    VERBATIM)
		add_custom_target(staple
		    DEPENDS notarize
		    COMMENT "Stapling notarization ticket to installer package..."
		    COMMAND xcrun stapler staple "${CMAKE_CURRENT_BINARY_DIR}/${CPACK_PACKAGE_FILE_NAME}.pkg"
		    VERBATIM)
	endif()

elseif (CMAKE_SYSTEM_NAME MATCHES "Windows")
	# Install WiX from https://wixtoolset.org/releases/
	list(APPEND CPACK_GENERATOR "WIX")
	set(CPACK_PACKAGE_INSTALL_DIRECTORY "NFIQ 2")

	if (CMAKE_SIZEOF_VOID_P MATCHES "8")
		# 64-bit GUID
		set(CPACK_WIX_UPGRADE_GUID "492101f0-5052-11eb-bb05-1b5668cc0087")
		set(CPACK_PACKAGE_NAME "${NFIQ2_PRODUCT_NAME} (x64)")
	else()
		#32-bit GUID
		set(CPACK_WIX_UPGRADE_GUID "226c3d8e-a622-fe6a-7c2d-3d3dc93eaacb")
		set(CPACK_PACKAGE_NAME "${NFIQ2_PRODUCT_NAME} (x86)")
	endif(CMAKE_SIZEOF_VOID_P MATCHES "8")

	# Required for building on NIST machines
	set(CPACK_WIX_CANDLE_EXTRA_FLAGS "-fips")

	# WiX doesn't allow non-integer versions
	if (NOT "${NFIQ2_VERSION_STATUS}" STREQUAL "")
		set(WIX_MAJOR ${NFIQ2_VERSION_MAJOR})
		set(WIX_MINOR ${NFIQ2_VERSION_MINOR})
		set(WIX_PATCH ${NFIQ2_VERSION_PATCH})
		set(WIX_STATUS "901")

		# Major version pre-release (3.0.0 -> 2.999.999.900)
		if (${WIX_MINOR} STREQUAL "0" AND ${WIX_PATCH} STREQUAL "0")
			math(EXPR WIX_MAJOR "${WIX_MAJOR} - 1")
			set(WIX_MINOR 999)
			set(WIX_PATCH 999)
		# Minor version pre-release (2.1.0 -> 2.0.999.900)
		elseif (${WIX_PATCH} STREQUAL "0" AND NOT ${WIX_MINOR} STREQUAL "0")
			math(EXPR WIX_MINOR "${WIX_MINOR} - 1")
			set(WIX_PATCH 999)
		# Patch version pre-release (2.1.2 -> 2.1.1.900)
		elseif (NOT ${WIX_PATCH} STREQUAL "0" AND NOT ${WIX_MINOR} STREQUAL "0")
			math(EXPR WIX_PATCH "${WIX_PATCH} - 1")
		endif()

		set(CPACK_PACKAGE_VERSION "${WIX_MAJOR}.${WIX_MINOR}.${WIX_PATCH}.${WIX_STATUS}")
		set(CPACK_PACKAGE_VERSION_MAJOR "${WIX_MAJOR}")
		set(CPACK_PACKAGE_VERSION_MINOR "${WIX_MINOR}")
		set(CPACK_PACKAGE_VERSION_PATCH "${WIX_PATCH}")
	endif()
	if (CMAKE_SIZEOF_VOID_P EQUAL 8)
		set(CPACK_SYSTEM_NAME "win64")
	else()
		set(CPACK_SYSTEM_NAME "win32")
	endif()
	set(CPACK_PACKAGE_FILE_NAME "nfiq2-${CPACK_PACKAGE_VERSION}-${CPACK_SYSTEM_NAME}")
elseif (EXISTS "/etc/debconf.conf")
	list(APPEND CPACK_GENERATOR "DEB")
	set(CPACK_DEB_COMPONENT_INSTALL ON)

	find_program(SHLIB dpkg-shlibdeps)
	if (SHLIB)
		set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
	else()
		message(AUTHOR_WARNING "dpkg-shlibdeps not found. Run `sudo apt install dpkg-dev` to properly calculate dependencies on Debian.")
		set(CPACK_DEBIAN_NFIQ2_CLI_PACKAGE_DEPENDS "libc6,libstdc++6,libgcc-s1,libzstd1,liblzma5,libjbig0,libwebp6,libssl1.1,libopenjp2-7,libjpeg-turbo8,libpng16-16,libtiff5,zlib1g,libdb5.3++,libsqlite3-0,liblzma5")
	endif()

	# We're customizing the package name, so we need to define these ourselves
	set(CPACK_DEBIAN_PACKAGE_RELEASE 1)
	execute_process(COMMAND dpkg --print-architecture OUTPUT_VARIABLE CPACK_DEBIAN_PACKAGE_ARCHITECTURE)
	string(STRIP ${CPACK_DEBIAN_PACKAGE_ARCHITECTURE} CPACK_DEBIAN_PACKAGE_ARCHITECTURE)

	set(CPACK_DEBIAN_DEVELOPMENT_FILE_NAME "nfiq2-dev_${CPACK_PACKAGE_VERSION}-${CPACK_DEBIAN_PACKAGE_RELEASE}_${CPACK_DEBIAN_PACKAGE_ARCHITECTURE}.deb")
	set(CPACK_DEBIAN_NFIQ2_CLI_FILE_NAME "nfiq2_${CPACK_PACKAGE_VERSION}-${CPACK_DEBIAN_PACKAGE_RELEASE}_${CPACK_DEBIAN_PACKAGE_ARCHITECTURE}.deb")

	# Spaces are not allowed
	set(CPACK_DEBIAN_DEVELOPMENT_PACKAGE_NAME "nfiq2-dev")
	set(CPACK_DEBIAN_NFIQ2_CLI_PACKAGE_NAME "nfiq2")

	set(CPACK_DEBIAN_NFIQ2_CLI_COMPONENT_DESCRIPTION "${CPACK_COMPONENT_NFIQ2_CLI_DESCRIPTION_SUMMARY}")
	set(CPACK_DEBIAN_DEVELOPMENT_COMPONENT_DESCRIPTION "${CPACK_COMPONENT_DEVELOPMENT_DESCRIPTION_SUMMARY}")
	set(CPACK_DEBIAN_NFIQ2_CLI_PACKAGE_CONTROL_STRICT_PERMISSION YES)
	set(CPACK_DEBIAN_DEVELOPMENT_PACKAGE_CONTROL_STRICT_PERMISSION YES)
elseif (EXISTS "/etc/redhat-release")
	set(CPACK_RPM_EXCLUDE_FROM_AUTO_FILELIST_ADDITION "/usr/local;/usr/local/bin")
	list(APPEND CPACK_GENERATOR "RPM")
	set(CPACK_RPM_COMPONENT_INSTALL ON)

	set(CPACK_RPM_MAIN_COMPONENT nfiq2_cli)
	set(CPACK_RPM_PACKAGE_NAME "nfiq2")

	# FIXME: Won't work, because we're installing "files"
	set(CPACK_RPM_PACKAGE_AUTOREQ YES)
	set(CPACK_RPM_NFIQ2_CLI_PACKAGE_AUTOREQ 1)
	set(CPACK_RPM_DEVELOPMENT_PACKAGE_AUTOREQ 1)
	set(CPACK_RPM_NFIQ2_CLI_PACKAGE_REQUIRES "openssl,openjpeg2,libjpeg-turbo,libpng,libtiff,zlib,libdb-cxx,sqlite,xz-lzma-compat")

	# We're customizing the package name, so we need to define these ourselves
	set(CPACK_RPM_PACKAGE_RELEASE 1)
	execute_process(COMMAND uname -m OUTPUT_VARIABLE CPACK_RPM_PACKAGE_ARCHITECTURE)
	string(STRIP ${CPACK_RPM_PACKAGE_ARCHITECTURE} CPACK_RPM_PACKAGE_ARCHITECTURE)

	# Get "Enterprise Linux" revision
	execute_process(COMMAND cat /etc/redhat-release OUTPUT_VARIABLE EL_REV)
	string(REGEX REPLACE "[^0-9]" "" EL_REV "${EL_REV}")
	string(SUBSTRING "${EL_REV}" 0 1 EL_REV)
	string(PREPEND EL_REV "el")

	# Spaces allowed, but look weird
	set(CPACK_RPM_DEVELOPMENT_PACKAGE_NAME "nfiq2-devel")
	set(CPACK_RPM_NFIQ2_CLI_PACKAGE_NAME "nfiq2")

	set(CPACK_RPM_NFIQ2_CLI_FILE_NAME "nfiq2-${CPACK_PACKAGE_VERSION}-${CPACK_RPM_PACKAGE_RELEASE}.${EL_REV}.${CPACK_RPM_PACKAGE_ARCHITECTURE}.rpm")
	set(CPACK_RPM_DEVELOPMENT_FILE_NAME "nfiq2-devel-${CPACK_PACKAGE_VERSION}-${CPACK_RPM_PACKAGE_RELEASE}.${EL_REV}.${CPACK_RPM_PACKAGE_ARCHITECTURE}.rpm")
endif()

include(CPack)
