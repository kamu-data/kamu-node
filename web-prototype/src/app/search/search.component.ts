import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {MatTableDataSource} from '@angular/material/table';
import {AppSearchService} from "./search.service";
import {
  PageInfoInterface,
  SearchHistoryInterface,
  SearchOverviewDatasetsInterface,
  SearchOverviewInterface
} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {searchAdditionalButtonsEnum} from "./search.interface";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";
import {Router} from "@angular/router";
import {query} from "@angular/animations";


@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public isMobileView: boolean = false;
  public searchValue: string = '';
  public currentPage: number = 1;
  public isMinimizeSearchAdditionalButtons: boolean = false;
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
    tableSource: SearchOverviewDatasetsInterface[],
    isResultQuantity: boolean,
    resultUnitText: string,
    isClickableRow: boolean,
    pageInfo: PageInfoInterface,
    totalCount: number
  };
  public searchData: SearchOverviewDatasetsInterface[] = [];

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
      private router: Router,
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService) {
  }

  public ngAfterContentInit(): void {
    this.tableData.tableSource = this.searchData
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }

    this.initTableData();
    this.onSearch("");
    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appSearchService.onSearchDataChanges.subscribe((data: SearchOverviewInterface) => {
      this.tableData.tableSource = data.dataset;
      this.tableData.pageInfo = data.pageInfo;
      this.tableData.totalCount = data.totalCount;
      this.currentPage = data.currentPage;
    });
  }

  private initTableData(): void {
    this.tableData = {
      tableSource: this.searchData,
      resultUnitText: 'dataset results',
      isResultQuantity: true,
      isClickableRow: true,
      pageInfo: {
        hasNextPage: false,
        hasPreviousPage: false,
        totalPages: 1
      },
      totalCount: 0
    };
  }

  public onPageChange(currentPage: number): void {
    this.currentPage = currentPage;
    this.onSearch(this.searchValue, currentPage - 1)
  }

  public onSelectDataset(id: string): void {
    this.router.navigate(['/dataset-view'], {queryParams: {id}});
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

  public onSearch(searchValue: string, page?: number): void {
    debugger
    this.appSearchService.search(searchValue, page);
  }

}