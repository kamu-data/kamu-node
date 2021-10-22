import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {DatasetInfoInterface, SearchHistoryInterface} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";
import {searchAdditionalButtonsEnum} from "../search/search.interface";
import {DatasetViewTypeEnum} from "./dataset-view.interface";
import {AppDatasetService} from "./dataset.service";
import {Router} from "@angular/router";

@Component({
  selector: 'app-dataset',
  templateUrl: './dataset.component.html',
  styleUrls: ['./dataset-view.component.sass']
})
export class DatasetComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public isMobileView: boolean = false;
  public datasetInfo: DatasetInfoInterface;
  public searchValue: string = '';
  public isMinimizeSearchAdditionalButtons: boolean = false;
  public datasetViewType: DatasetViewTypeEnum = DatasetViewTypeEnum.overview;
  public searchAdditionalButtonsData: SearchAdditionalButtonInterface[] = [{
    textButton: searchAdditionalButtonsEnum.Descission
  }, {
    textButton: searchAdditionalButtonsEnum.Reputation
  }, {
    textButton: searchAdditionalButtonsEnum.Explore,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }, {
    textButton: searchAdditionalButtonsEnum.DeriveForm,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }];

  // tslint:disable-next-line: no-any
  public tableData: {
    isTableHeader: boolean,
    displayedColumns?: any[],
    tableSource: any,
    isResultQuantity: boolean,
    isClickableRow: boolean
  };
  public searchData: SearchHistoryInterface[] = [];
  private _window: Window;

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.isMinimizeSearchAdditionalButtons = AppValues.isMobileView();
    this.isMobileView = AppValues.isMobileView();

    if (AppValues.isMobileView()) {
      this.sidenavService.close();
    } else {
      this.sidenavService.open();
    }
  }

  constructor(
      private appDatasetService: AppDatasetService,
      private sidenavService: SideNavService,
      private router: Router) {
    this._window = window;
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }
    this.initTableData();

    this.initDatasetViewByType();

    this.appDatasetService.onSearchDatasetInfoChanges.subscribe((info: DatasetInfoInterface) => {
      this.datasetInfo = info;
    })
    this.appDatasetService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    // tslint:disable-next-line: no-any
    this.appDatasetService.onSearchDataChanges.subscribe((data: any[]) => {
      this.tableData.tableSource = data;
    });
  }

  public ngAfterContentInit(): void {
    this.tableData.tableSource = this.searchData
  }

  private initTableData(): void {
    this.tableData = {
      isTableHeader: true,
      tableSource: this.searchData,
      isResultQuantity: true,
      isClickableRow: false
    };
  }
  public getResultUnitText(): string {
    const searchDataset: string = this.getDatasetId();
    return `results in ${searchDataset}`
  }

  private initDatasetViewByType(): void {
    let searchParams: string[] = this._window.location.search.split('&type=');

    if (searchParams.length > 1) {
      let type: DatasetViewTypeEnum = AppValues.fixedEncodeURIComponent(searchParams[1].split('&')[0]) as DatasetViewTypeEnum;

      if (type === DatasetViewTypeEnum.metadata) {
        this.onSearchMetadata();
      } else {
        this.onSearchDataset();
      }
    }
  }

  private getDatasetId(): string {
    let searchParams: string[] = this._window.location.search.split('?id=');

    if (searchParams.length > 1) {
      return AppValues.fixedEncodeURIComponent(searchParams[1].split('&')[0]);
    }
    return '';
  }

  public momentConverDatetoLocalWithFormat(date: string): string {
    return AppValues.momentConverDatetoLocalWithFormat({date: new Date(date), format: 'DD MMM YYYY', isTextDate: true});
  }


  public onClickSearchAdditionalButton(method: string) {
    if (method === searchAdditionalButtonsEnum.DeriveForm) {
      this.onClickDeriveForm();
    }
    if (method === searchAdditionalButtonsEnum.Reputation) {
      this.onClickReputation();
    }
    if (method === searchAdditionalButtonsEnum.Explore) {
      this.onClickExplore();
    }
    if (method === searchAdditionalButtonsEnum.Descission) {
      this.onClickDescission();
    }
  }

  private onClickDeriveForm() {
  }
  private onClickExplore() {
  }
  private onClickReputation() {
  }
  private onClickDescission() {
  }

  public onSearchMetadata(): void {
    this.router.navigate([AppValues.urlDatasetView], {queryParams: {id: this.getDatasetId(), type: AppValues.urlDatasetViewMetadataType}});

    this.datasetViewType = DatasetViewTypeEnum.metadata;
    this.appDatasetService.onSearchMetadata(this.getDatasetId());
  }

  public onSearchDataset(page: number = 0): void {
    this.router.navigate([AppValues.urlDatasetView], {queryParams: {id: this.getDatasetId(), type: AppValues.urlDatasetViewOverviewType}});

    this.datasetViewType = DatasetViewTypeEnum.overview;

    this.appDatasetService.searchDataset(this.getDatasetId(), page);
  }

  public onSearchLinageDataset(): void {
    this.router.navigate([AppValues.urlDatasetView], {queryParams: {id: this.getDatasetId(), type: DatasetViewTypeEnum.linage}});

    this.datasetViewType = DatasetViewTypeEnum.linage;
    this.appDatasetService.onSearchLinageDataset(this.getDatasetId());
  }
}
