import {AppSearchService} from './search.service';
import {
  PageInfoInterface,
  SearchOverviewDatasetsInterface,
  SearchOverviewInterface
} from '../interface/search.interface';
import AppValues from '../common/app.values';
import {searchAdditionalButtonsEnum} from './search.interface';
import {SearchAdditionalButtonInterface} from '../components/search-additional-buttons/search-additional-buttons.interface';
import {MatSidenav} from '@angular/material/sidenav';
import {SideNavService} from '../services/sidenav.service';
import {Router} from '@angular/router';
import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';


@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public isMobileView = false;
  public searchValue = '';
  public currentPage = 1;
  public isMinimizeSearchAdditionalButtons = false;
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
      private router: Router,
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService,
  ) {
      this._window = window;
  }

  public ngAfterContentInit(): void {
    this.tableData.tableSource = this.searchData;

    this.changePageAndSearch();
  }


  public ngOnInit(): void {
    if (this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }

    this.initTableData();

    this.changePageAndSearch();


    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
      this.onSearch(value, this.currentPage);
    });

    this.appSearchService.onSearchDataChanges.subscribe((data: SearchOverviewInterface) => {
      this.tableData.tableSource = data.dataset;
      this.tableData.pageInfo = data.pageInfo;
      this.tableData.totalCount = data.totalCount;
      this.currentPage = data.currentPage;
    });
  }
  private changePageAndSearch(): void {
    let page = 1;
    let currentId = '';

    if (this._window.location.search.split('?id=').length > 1) {
      currentId = this._window.location.search.split('?id=')[1].split('&')[0];
      this.searchValue = currentId;

      const searchPageParams: string[] = this._window.location.search.split('&p=');
      if (searchPageParams[1]) {
        page = Number(searchPageParams[1].split('&')[0]);
      }
    }

    this.currentPage = page;
    this.onSearch(currentId, page);
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

  public onPageChange(params: {currentPage: number, isClick: boolean}): void {
    this.currentPage = params.currentPage;

    this.router.navigate([AppValues.urlSearch], {queryParams: {id: this.searchValue, p: params.currentPage}});
  }

  public onSelectDataset(id: string): void {
    this.router.navigate([AppValues.defaultUsername, AppValues.urlDatasetView], {queryParams: {id, type: AppValues.urlDatasetViewOverviewType}});
  }


  public onSearch(searchValue: string, page: number = 1): void {
    this.appSearchService.search(searchValue, page - 1);
  }

}
