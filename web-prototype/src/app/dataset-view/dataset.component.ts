import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {SearchHistoryInterface} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";
import {searchAdditionalButtonsEnum} from "../search/search.interface";
import {DatasetViewTypeEnum} from "./dataset-view.interface";
import {AppDatasetService} from "./dataset.service";

const ELEMENT_DATA: SearchHistoryInterface[] = [];

@Component({
  selector: 'app-dataset',
  templateUrl: './dataset.component.html',
  styleUrls: ['./dataset-view.component.sass']
})
export class DatasetComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public isMobileView: boolean = false;
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
      private sidenavService: SideNavService) {
    this._window = window;
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }
    this.initTableData();

    this.onSearchDataset();

    this.appDatasetService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appDatasetService.onSearchDataChanges.subscribe((data: any[]) => {
      this.tableData.tableSource = data;
    });
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

  private getDatasetId(): string {
    return this._window.location.search.split('=')[1];
  }

  public ngAfterContentInit(): void {
    this.tableData.tableSource = this.searchData
  }
  public onSelectDataset(dataset: any): void {
    return;
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
    this.datasetViewType = DatasetViewTypeEnum.metadata;
    this.appDatasetService.onSearchMetadata(this.getDatasetId());
  }

  public onSearchDataset(page: number = 0): void {
    this.datasetViewType = DatasetViewTypeEnum.overview;
    this.appDatasetService.searchDataset(this.getDatasetId(), page);
  }

  public onSearchLinageDataset(): void {
    this.datasetViewType = DatasetViewTypeEnum.linage;
    this.appDatasetService.onSearchLinageDataset(this.getDatasetId());
  }
}