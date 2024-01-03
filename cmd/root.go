package cmd

import (
	"archive/zip"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"

	bufra "github.com/avvmoto/buf-readerat"
	retryablehttp "github.com/hashicorp/go-retryablehttp"
	"github.com/snabb/httpreaderat"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.PersistentFlags().BoolP("list", "l", false, "list files in zip")
}

var rootCmd = &cobra.Command{
	Use:   "zcat [flags] [file/url] file [...file]",
	Short: "zcat",
	Args:  cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		r, size, err := open(args[0])
		if err != nil {
			return err
		}
		zipReader, err := zip.NewReader(r, *size)
		if err != nil {
			return err
		}
		if l, _ := cmd.Flags().GetBool("list"); l {
			err = list_zip_contents(zipReader)
		} else {
			err = cat_zip_file(zipReader, args[1:])
		}
		return err
	},
}

func open(path string) (io.ReaderAt, *int64, error) {
	r, size, err := openLocalFile(path)
	if err != nil {
		return openRemoteFile(path)
	} else {
		return r, size, err
	}
}

func openLocalFile(path string) (io.ReaderAt, *int64, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, nil, err
	}
	fi, err := f.Stat()
	if err != nil {
		f.Close()
		return nil, nil, err
	}
	size := fi.Size()
	return f, &size, nil
}

func openRemoteFile(url string) (io.ReaderAt, *int64, error) {
	retryClient := retryablehttp.NewClient()
	retryClient.Logger = nil
	retryClient.HTTPClient.Timeout = 7 * time.Second
	retryClient.RetryMax = 5
	standardClient := retryClient.StandardClient()

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, nil, err
	}

	htrdr, err := httpreaderat.New(standardClient, req, nil)
	if err != nil {
		return nil, nil, err
	}
	bhtrdr := bufra.NewBufReaderAt(htrdr, 1024*4)

	size := htrdr.Size()
	return bhtrdr, &size, nil
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func list_zip_contents(r *zip.Reader) error {
	fmt.Printf("%9s  %10s %5s   %4s\n", "Length", "Date", "Time", "Name")
	fmt.Printf("%9s  %10s %5s   %4s\n",
		strings.Repeat("-", 9),
		strings.Repeat("-", 10),
		strings.Repeat("-", 5),
		strings.Repeat("-", 4),
	)
	var totalLen uint64 = 0
	totalCnt := 0
	for _, f := range r.File {
		fmt.Printf("%9d  %10s %5s   %4s\n",
			f.UncompressedSize64,
			f.Modified.Format("2006-01-02"),
			f.Modified.Format("15:04"),
			f.Name,
		)
		totalLen += f.UncompressedSize64
		totalCnt++
	}
	fmt.Printf("%s%s%s\n",
		strings.Repeat("-", 9),
		strings.Repeat(" ", 21),
		strings.Repeat("-", 7),
	)
	fmt.Printf("%9d%s%d files\n",
		totalLen,
		strings.Repeat(" ", 21),
		totalCnt,
	)

	return nil
}

func cat_zip_file(r *zip.Reader, file_list []string) error {
	fileSet := make(map[string]*zip.File)
	for _, name := range file_list {
		fileSet[name] = nil
	}
	for _, f := range r.File {
		if _, ok := fileSet[f.Name]; ok {
			fileSet[f.Name] = f
		}
	}
	for _, file_name := range file_list {
		if f, ok := fileSet[file_name]; ok {
			rc, err := f.Open()
			if err != nil {
				return err
			}
			defer rc.Close()
			if len(file_list) > 1 {
				fmt.Printf("name: %s\n", f.Name)
			}
			_, err = io.Copy(os.Stdout, rc)
			if err != nil {
				return err
			}
		}
	}
	return nil
}
