#ifndef NFIQ2_QUALITYMODULES_IMGPROCROI_H_
#define NFIQ2_QUALITYMODULES_IMGPROCROI_H_

#include <nfiq2_constants.hpp>
#include <nfiq2_fingerprintimagedata.hpp>
#include <opencv2/core.hpp>
#include <quality_modules/Module.h>

#include <string>
#include <vector>

namespace NFIQ2 { namespace QualityMeasures {

class ImgProcROI : public Algorithm {
    public:
	struct ImgProcROIResults {
		/** input block size in pixels */
		unsigned int chosenBlockSize {};
		/**
		 * overall number of complete blocks (with full block size)
		 * in the image
		 */
		unsigned int noOfCompleteBlocks {};
		/** overall number of blocks in the image */
		unsigned int noOfAllBlocks {};
		/** detected ROI blocks with position and size */
		std::vector<cv::Rect> vecROIBlocks {};
		/** number of ROI pixels detected in the image (not blocks) */
		unsigned int noOfROIPixels {};
		/** number of pixels of the image */
		unsigned int noOfImagePixels {};
		/** mean of all grayvalues of all ROI pixels */
		double meanOfROIPixels {};
		/** standard deviation of all grayvalues of all ROI pixels */
		double stdDevOfROIPixels {};
	};

	ImgProcROI(const NFIQ2::FingerprintImageData &fingerprintImage);
	virtual ~ImgProcROI();

	std::string getName() const override;

	static std::vector<std::string> getNativeQualityMeasureIDs();

	static ImgProcROIResults computeROI(cv::Mat &img, unsigned int bs);

	/** @throw NFIQ2::Exception
	 * Img Proc Results could not be computed.
	 */
	ImgProcROIResults getImgProcResults();

    private:
	std::unordered_map<std::string, double> computeFeatureData(
	    const NFIQ2::FingerprintImageData &fingerprintImage);

	ImgProcROIResults imgProcResults_ {};
	bool imgProcComputed_ { false };
	static bool isBlackPixelAvailable(cv::Mat &img, cv::Point &point);
};

}}

#endif /* NFIQ2_QUALITYMODULES_IMGPROCROI_H_ */

/******************************************************************************/
