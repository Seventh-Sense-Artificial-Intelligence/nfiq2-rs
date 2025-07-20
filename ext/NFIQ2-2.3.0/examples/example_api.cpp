// Universal include for all NFIQ2 headers
#include <nfiq2.hpp>

#include <iostream>
#include <memory>

// Parses PGM Files. Used to extract data from provided sythetic fingerprint
// images.
void
parsePGM(char *filename, std::shared_ptr<uint8_t> &data, uint32_t &rows,
    uint32_t &cols)
{
	// Open PGM file.
	std::ifstream input(filename, std::ios::binary);
	if (!input.is_open()) {
		std::cerr << "Cannot open image: " << filename << "\n";
		return;
	}

	// Read in magic number.
	std::string magicNumber;
	input >> magicNumber;
	if (magicNumber != "P5") {
		std::cerr << "Error reading magic number from file."
			  << "\n";
		return;
	}

	uint16_t maxValue;
	// Read in image header values - cols, rows.
	// Ignoring the result of maxValue as it is not needed.
	input >> cols >> rows >> maxValue;
	if (!input.good()) {
		std::cerr
		    << "Error, premature end of file while reading header."
		    << "\n";
		return;
	}
	uint32_t size = cols * rows;

	// Skip line break.
	input.ignore(std::numeric_limits<std::streamsize>::max(), '\n');

	// Allocating new array on heap.
	uint8_t *dataHeap = new uint8_t[size];
	data.reset(dataHeap, std::default_delete<uint8_t[]>());

	// Read in raw pixel data.
	input.read((char *)data.get(), size);
	if (!input.good()) {
		std::cerr << "Error, only read " << input.gcount() << " bytes."
			  << "\n";
		return;
	}
	return;
}

void
printUsage()
{
	std::cout << "example_api: usage: example_api [-h] "
#ifndef NFIQ2_EMBEDDED_MODEL
		     "modelInfoFile "
#endif
		     "fingerPrintImage\n";
}

void
printHelp()
{
	std::cout << "NFIQ 2 API Example\n\nThis is a sample program that "
		     "shows how to use the NFIQ 2 API on an image.\n\n"
#if defined(NFIQ2_EMBEDDED_MODEL)
		     "This command line tool takes one argument.\nIt is "
		     "the path to a single PGM fingerprint image.\n\n"
		     "Please provide arguments to the binary in the designated "
		     "order.\nEx: $ example_api fingerImage01.pgm\n";
#else
		     "This command line tool takes two arguments.\nThe first "
		     "is the path to a NFIQ 2 RandomForest model information "
		     "file.\nThe second is the path to a single PGM "
		     "fingerprint image.\n\n"
		     "Please provide arguments to the binary in the designated "
		     "order.\nEx: $ example_api nist_plain_tir.txt "
		     "fingerImage01.pgm\n";
#endif
}

int
main(int argc, char **argv)
{
	const std::string helpStr { "-h" };

#if defined(NFIQ2_EMBEDDED_MODEL)
	if (argc != 2) {
		printUsage();
		return (EXIT_FAILURE);
	}

	if (helpStr.compare(argv[1]) == 0) {
		printHelp();
		return (EXIT_FAILURE);
	}

	static const size_t pgmPathArgvIdx { 1 };
#else
	if (argc == 2 && helpStr.compare(argv[1]) == 0) {
		printHelp();
		return (EXIT_FAILURE);
	}

	if (argc != 3) {
		printUsage();
		return (EXIT_FAILURE);
	}

	static const size_t pgmPathArgvIdx { 2 };
#endif

	// Note: NFIQ 2 only operates on images captured at a resolution of
	// 500 PPI. Unfortunately, the images contained within this repository
	// are encoded in PGM format, which does not encode resolution
	// information. ALWAYS make use of image libraries to decode image
	// resolution information and convert to PPI to avoid erroneous NFIQ 2
	// results.
	static const uint16_t PPI = 500;

	// You must instantiate an NFIQ2::Algorithm instance, which requires
	// a model. The build system supports embedding a single model within
	// the library or reading a model at runtime.
#if defined(NFIQ2_EMBEDDED_MODEL)
	// If you've built NFIQ 2 with a model embedded, there's nothing to do
	// other than instantiate the object.
	NFIQ2::Algorithm model {};
	if (!model.isEmbedded()) {
		std::cerr << "Model is not embedded.\n";
		return (EXIT_FAILURE);
	}
#else
	// To compute an NFIQ 2 score, you must load the RF model you wish to
	// use. This segment loads the model information file that contains data
	// about the RF model.
	NFIQ2::ModelInfo modelInfoObj {};
	try {
		modelInfoObj = NFIQ2::ModelInfo(argv[1]);
	} catch (...) {
		std::cerr
		    << "Could not parse model info file. "
		       "Ensure it is the first argument on the cmd line\n";
		printUsage();
		return (EXIT_FAILURE);
	}

	// This segment loads the model into memory so that it can be used to
	// generate a NFIQ 2 score.
	NFIQ2::Algorithm model {};
	try {
		model = NFIQ2::Algorithm(modelInfoObj);
	} catch (...) {
		std::cerr
		    << "Could not initialize model from model info file\n";
		return (EXIT_FAILURE);
	}
#endif

	// This calls the PGM parser on the image passed by command line
	// argument.
	uint32_t rows = 0;
	uint32_t cols = 0;
	std::shared_ptr<uint8_t> data {};
	parsePGM(argv[pgmPathArgvIdx], data, rows, cols);

	// This constructs a FingerprintImageData object that stores the
	// relevant image information NFIQ 2 needs to compute a score.
	NFIQ2::FingerprintImageData rawImage = NFIQ2::FingerprintImageData(
	    data.get(), cols * rows, cols, rows, 0, PPI);

	// Calculate all native quality measures.
	std::vector<std::shared_ptr<NFIQ2::QualityMeasures::Algorithm>>
	    algorithms {};
	try {
		algorithms = NFIQ2::QualityMeasures::
		    computeNativeQualityMeasureAlgorithms(rawImage);
	} catch (const NFIQ2::Exception &e) {
		std::cerr << "Error in calculating quality features: "
			  << e.what() << '\n';
		return (EXIT_FAILURE);
	}

	// Pass the native quality measures through the random forest to obtain
	// a unified quality score
	unsigned int nfiq2 {};
	try {
		nfiq2 = model.computeUnifiedQualityScore(algorithms);
	} catch (...) {
		std::cerr << "Error in calculating NFIQ 2 score\n";
		return (EXIT_FAILURE);
	}

	// The output generated by this sample app is detailed in the
	// README file. The following code iterates through
	// the NFIQ2Results object, extracts scores, and prints them to
	// stdout.
	std::cout << "UnifiedQualityScore: " << nfiq2 << '\n';

	// Actionable Feedback
	std::vector<std::string> actionableIDs =
	    NFIQ2::QualityMeasures::getActionableQualityFeedbackIDs();

	std::unordered_map<std::string, double> actionableQuality =
	    NFIQ2::QualityMeasures::getActionableQualityFeedback(algorithms);

	for (const auto &actionableID : actionableIDs) {
		std::cout << actionableID << ": "
			  << actionableQuality.at(actionableID) << '\n';
	}

	// Quality Feature Values
	std::vector<std::string> featureIDs =
	    NFIQ2::QualityMeasures::getNativeQualityMeasureIDs();

	std::unordered_map<std::string, double> qualityFeatures =
	    NFIQ2::QualityMeasures::getNativeQualityMeasures(algorithms);

	for (const auto &featureID : featureIDs) {
		std::cout << featureID << ": " << qualityFeatures.at(featureID)
			  << '\n';
	}

	// Image Processed
	return (EXIT_SUCCESS);
}
