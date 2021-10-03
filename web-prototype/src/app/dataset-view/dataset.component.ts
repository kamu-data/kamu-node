import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {MatTableDataSource} from '@angular/material/table';
import {SearchHistoryInterface} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";
import {AppSearchService} from "../search/search.service";
import {searchAdditionalButtonsEnum} from "../search/search.interface";
import {DatasetViewTypeEnum} from "./dataset-view.interface";

const ELEMENT_DATA: SearchHistoryInterface[] = [];

@Component({
  selector: 'app-dataset',
  templateUrl: './dataset.component.html',
  styleUrls: ['./dataset-view.component.sass']
})
export class DatasetComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public typeSelected: string = 'overview'
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
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService) {
    this._window = window;
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }
    this.initTableData();

    this.appSearchService.searchHistory();

    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appSearchService.onSearchDataChanges.subscribe((data: any[]) => {
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
    debugger
    const searchDataset: string = this._window.location.search.split('=')[1];
    return `results in ${searchDataset}`
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

  public onSearchProjections(): void {
    this.appSearchService.searchLastTenFields();
  }
  public onSearchMetadata(): void {
    this.appSearchService.onSearchMetadata();
  }
}